mod sentry;

use sentry::{Issue, Job, Triage};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use squadron_extension_api::{Account, Agent, Extension, Request, Response, host, squadron_extension};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

struct Sentry;

const HOUR: u64 = 3_600_000;

fn en() -> bool {
    host::state_get("lang").as_deref() == Some("en")
}

macro_rules! t {
    ($fr:literal, $en:literal $(, $a:expr)*) => {
        if en() { format!($en $(, $a)*) } else { format!($fr $(, $a)*) }
    };
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Follow {
    folder: String,
    account: String,
    project_id: String,
    slug: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Proposal {
    status: String,
    text: String,
    at: u64,
}

struct Acc {
    id: String,
    host: String,
    org: String,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

fn get<T: DeserializeOwned + Default>(key: &str) -> T {
    host::state_get(key).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn put<T: Serialize>(key: &str, value: &T) {
    host::state_set(key, serde_json::to_string(value).ok().as_deref());
}

fn option(id: &str) -> Value {
    serde_json::from_str::<Value>(&host::options()).unwrap_or_default()[id].clone()
}

fn field(fields: &[(String, String)], name: &str) -> String {
    fields.iter().find(|(k, _)| k == name).map(|(_, v)| v.trim().to_string()).unwrap_or_default()
}

fn accounts() -> Vec<Acc> {
    host::accounts().into_iter().map(|a| Acc { host: field(&a.fields, "host"), org: field(&a.fields, "org"), id: a.id }).collect()
}

fn account(id: &str) -> Result<Acc, String> {
    accounts().into_iter().find(|a| a.id == id).ok_or_else(|| t!("Ce compte Sentry n’existe plus.", "This Sentry account no longer exists."))
}

fn encode(text: &str) -> String {
    text.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

fn request(id: &str, host_name: &str, method: &str, path: &str, body: Option<String>) -> Request {
    Request {
        method: method.into(),
        url: format!("https://{host_name}/api/0/{path}"),
        headers: vec![("Authorization".into(), format!("Bearer {{{{{id}.token}}}}")), ("Content-Type".into(), "application/json".into())],
        body,
    }
}

fn call(id: &str, host_name: &str, method: &str, path: &str, body: Option<String>) -> Result<Value, String> {
    reply(host::fetch(&request(id, host_name, method, path, body))?, path)
}

fn reply(r: Response, path: &str) -> Result<Value, String> {
    match r.status {
        200..=299 => Ok(serde_json::from_str(&r.body).unwrap_or(Value::Null)),
        401 | 403 => Err(t!("Sentry refuse ce jeton (code {}) : vérifiez ses droits.", "Sentry refused this token ({}): check its scopes.", r.status)),
        404 => Err(t!("Sentry ne trouve pas {} : vérifiez l’organisation.", "Sentry cannot find {}: check the organization.", path)),
        s => Err(t!("Sentry a répondu {} pour {}.", "Sentry answered {} for {}.", s, path)),
    }
}

fn org_path(acc: &Acc, path: &str) -> String {
    format!("organizations/{}/{path}", encode(&acc.org))
}

fn api(acc: &Acc, method: &str, path: &str, body: Option<String>) -> Result<Value, String> {
    call(&acc.id, &acc.host, method, &org_path(acc, path), body)
}

fn api_many(acc: &Acc, method: &str, paths: &[String], body: Option<String>) -> Vec<Result<Value, String>> {
    let requests: Vec<Request> = paths.iter().map(|p| request(&acc.id, &acc.host, method, &org_path(acc, p), body.clone())).collect();
    host::fetch_many(&requests).into_iter().zip(paths).map(|(r, p)| r.and_then(|r| reply(r, p))).collect()
}

fn list(acc: &Acc, project_ids: &[&str]) -> Result<Vec<Issue>, String> {
    search(acc, project_ids, "is:unresolved", "14d", 50, "date").map(|(l, _)| l)
}

fn sort_of(sort: &str) -> &str {
    if ["date", "new", "freq", "user"].contains(&sort) { sort } else { "date" }
}

// Sentry gives the number of matching issues in X-Hits, beyond the page it returns.
fn search(acc: &Acc, project_ids: &[&str], query: &str, period: &str, limit: u32, sort: &str) -> Result<(Vec<Issue>, u64), String> {
    let mut path = format!("issues/?query={}&sort={}&limit={limit}&statsPeriod={period}", encode(query), sort_of(sort));
    for p in project_ids {
        path.push_str(&format!("&project={}", encode(p)));
    }
    for env in option("environments").as_str().unwrap_or_default().split(',').map(str::trim).filter(|e| !e.is_empty()) {
        path.push_str(&format!("&environment={}", encode(env)));
    }
    let r = host::fetch(&request(&acc.id, &acc.host, "GET", &org_path(acc, &path), None))?;
    let hits = r.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("x-hits")).and_then(|(_, v)| v.trim().parse().ok());
    let list = sentry::issues(&reply(r, &path)?.to_string());
    let total = hits.unwrap_or(list.len() as u64);
    Ok((list, total))
}

fn follows() -> Vec<Follow> {
    get("follows")
}

// Haiku, without tools, sorts the open errors: same-cause groups and errors with nothing to fix.
fn triage(acc: &Acc, found: &[Issue], force: bool) -> Triage {
    let key = format!("triage:{}", acc.id);
    let old: Triage = get(&key);
    let mut ids: Vec<String> = found.iter().map(|i| i.id.clone()).collect();
    ids.sort();
    if found.is_empty() || (!force && ids == old.ids && now().saturating_sub(old.at) < HOUR) {
        return old;
    }
    let facts_key = format!("facts:{}", acc.id);
    let mut facts: BTreeMap<String, String> = get(&facts_key);
    let missing: Vec<&Issue> = found.iter().filter(|i| !facts.contains_key(&i.id)).collect();
    let paths: Vec<String> = missing.iter().map(|i| format!("issues/{}/events/latest/", encode(&i.id))).collect();
    for (i, r) in missing.iter().zip(api_many(acc, "GET", &paths, None)) {
        if let Ok(event) = r {
            facts.insert(i.id.clone(), sentry::facts(&event));
        }
    }
    facts.retain(|k, _| ids.contains(k));
    put(&facts_key, &facts);
    let lines: Vec<String> = found.iter().map(|i| sentry::brief(i, facts.get(&i.id).map_or("", String::as_str))).collect();
    let input = format!("{}\n{}", sentry::triage_prompt(!en(), &sentry::iso(now())), lines.join("\n"));
    let args: Vec<String> = ["-p", "--model", "haiku", "--tools", "", "--restricted", "--strict-mcp-config", "--no-session-persistence", "--append-system-prompt", sentry::triage_system(!en())].iter().map(|s| s.to_string()).collect();
    let mut t = Triage { ids, at: now(), groups: old.groups, noise: old.noise, error: String::new() };
    match host::exec_input("claude", &args, None, &input) {
        Ok(o) if o.code == 0 => match sentry::read_triage(&o.stdout, found) {
            Some((groups, noise)) => (t.groups, t.noise) = (groups, noise),
            None => t.error = t!("réponse illisible du modèle", "unreadable answer from the model"),
        },
        Ok(o) => t.error = first_lines(&format!("{}\n{}", o.stderr, o.stdout)),
        Err(e) => t.error = e,
    }
    put(&key, &t);
    t
}

// Sentry projects are polled once per account, whichever Squadron projects follow them.
fn poll() {
    let started = now();
    let all = follows();
    let mut fresh_count = 0;
    let mut sorted = false;
    let mut queue: Vec<Job> = get("queue");
    for acc in accounts() {
        let mine: Vec<&Follow> = all.iter().filter(|f| f.account == acc.id).collect();
        if mine.is_empty() {
            continue;
        }
        let mut ids: Vec<&str> = mine.iter().map(|f| f.project_id.as_str()).collect();
        ids.sort();
        ids.dedup();
        let Ok(found) = list(&acc, &ids) else { continue };
        let t = triage(&acc, &found, false);
        sorted |= t.at >= started;
        let seen: Vec<String> = get(&format!("seen:{}", acc.id));
        let regressed: Vec<String> = get(&format!("regressed:{}", acc.id));
        let mut primed: Vec<String> = get(&format!("primed:{}", acc.id));
        let f = sentry::fresh(&found, &seen, &regressed, |p| primed.iter().any(|x| x == p));
        put(&format!("seen:{}", acc.id), &f.seen);
        put(&format!("regressed:{}", acc.id), &f.regressed);
        for id in &ids {
            if !primed.iter().any(|x| x == id) {
                primed.push(id.to_string());
            }
        }
        put(&format!("primed:{}", acc.id), &primed);
        for (issue, back) in f.found {
            let Some(follow) = mine.iter().find(|m| m.project_id == issue.project_id) else { continue };
            fresh_count += 1;
            let noise = t.noise.iter().any(|n| n.id == issue.id);
            let what = match (back, noise) {
                (_, true) => t!("Probablement sans suite", "Probably nothing to fix"),
                (true, _) => t!("Revenue", "Back"),
                _ => t!("Nouvelle erreur", "New error"),
            };
            host::notify(&format!("Sentry · {}", issue.project), &format!("{what} · {} · {}", issue.short_id, issue.title));
            if option("auto") != false && !noise {
                sentry::enqueue(&mut queue, Job { account: acc.id.clone(), id: issue.id.clone(), folder: follow.folder.clone(), regressed: back });
            }
        }
    }
    put("queue", &queue);
    if fresh_count > 0 {
        let unread: u64 = get::<u64>("unread") + fresh_count;
        put("unread", &unread);
        host::set_badge("sentry", Some(&unread.to_string()));
    }
    if fresh_count > 0 || sorted {
        post(json!({ "type": "changed", "to": "*" }));
    }
}

fn proposal_key(account: &str, id: &str) -> String {
    format!("prop:{account}:{id}")
}

fn save_proposal(account: &str, id: &str, p: &Proposal) {
    let key = proposal_key(account, id);
    let mut index: Vec<String> = get("props");
    index.retain(|k| k != &key);
    index.push(key.clone());
    while index.len() > 60 {
        host::state_set(&index.remove(0), None);
    }
    put("props", &index);
    put(&key, p);
    post(json!({ "type": "proposal", "to": "*", "account": account, "id": id, "proposal": p }));
}

fn details(acc: &Acc, id: &str) -> Result<(Value, Value), String> {
    let issue = api(acc, "GET", &format!("issues/{}/", encode(id)), None)?;
    let event = api(acc, "GET", &format!("issues/{}/events/latest/", encode(id)), None).or_else(|_| call(&acc.id, &acc.host, "GET", &format!("issues/{}/events/latest/", encode(id)), None))?;
    Ok((issue, event))
}

fn first_lines(text: &str) -> String {
    text.lines().filter(|l| !l.trim().is_empty()).take(6).collect::<Vec<_>>().join("\n")
}

// Claude Code reads the repository with read-only tools and writes a diagnosis and a diff.
fn propose(job: &Job) -> Proposal {
    let mut p = Proposal { status: "running".into(), text: String::new(), at: now() };
    save_proposal(&job.account, &job.id, &p);
    let result = account(&job.account).and_then(|acc| {
        let (issue, event) = details(&acc, &job.id)?;
        let context = sentry::context(&issue, &event);
        let model = option("model").as_str().unwrap_or("sonnet").to_string();
        let args: Vec<String> = ["-p", "--model", &model, "--tools", "Read,Grep,Glob", "--append-system-prompt", sentry::system(!en()), sentry::prompt(!en())].iter().map(|s| s.to_string()).collect();
        let out = host::exec_input("claude", &args, Some(&job.folder), &context)?;
        if out.code == 0 && !out.stdout.trim().is_empty() {
            Ok((out.stdout.trim().to_string(), sentry::issue(&issue)))
        } else {
            Err(first_lines(&format!("{}\n{}", out.stderr, out.stdout)))
        }
    });
    p.at = now();
    match result {
        Ok((text, issue)) => {
            p.status = "ready".into();
            p.text = text;
            host::notify(&t!("Correction proposée · {}", "Fix proposed · {}", issue.short_id), &issue.title);
        }
        Err(e) => {
            p.status = "failed".into();
            p.text = e;
        }
    }
    save_proposal(&job.account, &job.id, &p);
    p
}

fn minutes() -> u64 {
    option("interval").as_u64().unwrap_or(5).clamp(1, 60)
}

fn tick() {
    if now().saturating_sub(get::<u64>("polled")) >= minutes() * 60_000 - 5_000 {
        put("polled", &now());
        poll();
    }
    let mut queue: Vec<Job> = get("queue");
    if !queue.is_empty() {
        let job = queue.remove(0);
        put("queue", &queue);
        propose(&job);
    }
    let waiting = !get::<Vec<Job>>("queue").is_empty();
    host::set_timer(if waiting { 10 } else { (minutes() * 60) as u32 });
}

fn post(message: Value) {
    host::panel_post("main", &message.to_string());
}

fn issues_of(folder: &str, sort: &str) -> (Vec<Value>, Vec<String>, u64) {
    let mine: Vec<Follow> = follows().into_iter().filter(|f| f.folder == folder).collect();
    let mut out = Vec::new();
    let mut errors = Vec::new();
    let mut total = 0;
    for acc in accounts() {
        let ids: Vec<&str> = mine.iter().filter(|f| f.account == acc.id).map(|f| f.project_id.as_str()).collect();
        if ids.is_empty() {
            continue;
        }
        match search(&acc, &ids, "is:unresolved", "14d", 100, sort) {
            Ok((list, hits)) => {
                total += hits;
                for i in list {
                    let p: Option<Proposal> = host::state_get(&proposal_key(&acc.id, &i.id)).and_then(|s| serde_json::from_str(&s).ok());
                    out.push(json!({ "account": acc.id, "issue": i, "proposal": p }));
                }
            }
            Err(e) => errors.push(format!("{} · {}: {e}", acc.host, acc.org)),
        }
    }
    (out, errors, total)
}

fn init(agent: &Agent, sort: &str) -> Value {
    let accs: Vec<Value> = accounts().iter().map(|a| json!({ "id": a.id, "label": format!("{} · {}", a.org, a.host) })).collect();
    let mine: Vec<Follow> = follows().into_iter().filter(|f| f.folder == agent.project).collect();
    let (issues, errors, total) = issues_of(&agent.project, sort);
    let triage = sorted(&issues);
    json!({ "type": "init", "accounts": accs, "follows": mine, "issues": issues, "total": total, "sort": sort_of(sort), "errors": errors, "triage": triage, "days": days(&agent.project), "auto": option("auto") != false })
}

fn days(folder: &str) -> u64 {
    host::state_get(&format!("days:{folder}")).and_then(|s| s.parse().ok()).unwrap_or(30)
}

// Unresolved errors of this project not seen for `days`; Sentry keeps 90 days of events.
fn stale(folder: &str, days: u64) -> Result<Value, String> {
    let days = days.clamp(1, 89);
    host::state_set(&format!("days:{folder}"), Some(&days.to_string()));
    let mine: Vec<Follow> = follows().into_iter().filter(|f| f.folder == folder).collect();
    let mut out = Vec::new();
    for acc in accounts() {
        let ids: Vec<&str> = mine.iter().filter(|f| f.account == acc.id).map(|f| f.project_id.as_str()).collect();
        if ids.is_empty() {
            continue;
        }
        let found = search(&acc, &ids, &format!("is:unresolved lastSeen:+{days}d"), "90d", 100, "date")?.0;
        out.extend(found.into_iter().map(|i| json!({ "account": acc.id, "issue": i })));
    }
    Ok(json!({ "type": "stale", "days": days, "issues": out }))
}

// The triage of each account, cut to the issues this panel shows.
fn sorted(issues: &[Value]) -> Value {
    let shown = |acc: &str, id: &str| issues.iter().any(|v| v["account"] == acc && v["issue"]["id"] == id);
    let (mut at, mut groups, mut noise, mut errors) = (0u64, Vec::new(), Vec::new(), Vec::new());
    for acc in accounts() {
        if !issues.iter().any(|v| v["account"] == acc.id.as_str()) {
            continue;
        }
        let t: Triage = get(&format!("triage:{}", acc.id));
        at = at.max(t.at);
        if !t.error.is_empty() {
            errors.push(t.error);
        }
        for g in t.groups {
            let ids: Vec<String> = g.ids.into_iter().filter(|id| shown(&acc.id, id)).collect();
            if ids.len() >= 2 {
                groups.push(json!({ "account": acc.id, "subject": g.subject, "project": g.project, "ids": ids }));
            }
        }
        noise.extend(t.noise.into_iter().filter(|n| shown(&acc.id, &n.id)).map(|n| json!({ "account": acc.id, "id": n.id, "reason": n.reason, "why": n.why })));
    }
    json!({ "at": at, "groups": groups, "noise": noise, "errors": errors })
}

fn projects() -> Value {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    for acc in accounts() {
        match api(&acc, "GET", "projects/", None) {
            Ok(v) => out.extend(v.as_array().into_iter().flatten().map(|p| json!({ "account": acc.id, "id": p["id"], "slug": p["slug"], "name": p["name"] }))),
            Err(e) => errors.push(format!("{} · {}: {e}", acc.host, acc.org)),
        }
    }
    json!({ "type": "projects", "projects": out, "errors": errors })
}

fn draft(agent: &Agent, account_id: &str, id: &str) -> Result<(), String> {
    let acc = account(account_id)?;
    let (issue, event) = details(&acc, id)?;
    let i = sentry::issue(&issue);
    let proposal: Option<Proposal> = host::state_get(&proposal_key(account_id, id)).and_then(|s| serde_json::from_str(&s).ok());
    let ready = proposal.filter(|p| p.status == "ready").map(|p| p.text);
    let text = match ready {
        Some(p) => t!("Corrige l’erreur Sentry {} : {}\n\nUne première analyse propose ceci, à vérifier avant de l’appliquer :\n\n{}", "Fix the Sentry error {}: {}\n\nA first analysis proposes this, to check before applying it:\n\n{}", i.short_id, i.title, p),
        None => t!("Corrige l’erreur Sentry {} : {}\n\nLa stack trace est jointe. Trouve la cause, corrige-la et ajoute un test qui la couvre si c’est possible.", "Fix the Sentry error {}: {}\n\nThe stack trace is attached. Find the cause, fix it and add a test that covers it when possible.", i.short_id, i.title),
    } + &t!("\n\nCommence par me présenter ton plan, et attends mon accord avant de modifier le moindre fichier.", "\n\nStart by showing me your plan, and wait for my go-ahead before changing any file.");
    let title: String = i.title.chars().take(60).collect();
    let d = json!({
        "name": format!("{} {title}", i.short_id),
        "text": text,
        "goal": t!("L’erreur {} ne se reproduit plus et un test la couvre quand c’est possible.", "Error {} no longer happens and a test covers it when possible.", i.short_id),
        "context": format!("<ticket source=\"sentry\" key=\"{}\">\n{}\n</ticket>", i.short_id, sentry::context(&issue, &event)),
        "link": { "key": i.short_id, "title": i.title, "url": i.permalink, "tone": "err" },
    });
    host::open_new_agent(&agent.id, &d.to_string());
    Ok(())
}

fn answer(kind: &str, agent: &Agent, m: &Value) -> Result<Value, String> {
    let text = |k: &str| m[k].as_str().unwrap_or_default().to_string();
    let sort = text("sort");
    Ok(match kind {
        "ready" => {
            if let Some(lang) = m["lang"].as_str() {
                host::state_set("lang", Some(if lang.starts_with("fr") { "fr" } else { "en" }));
            }
            put("unread", &0u64);
            host::set_badge("sentry", None);
            init(agent, &sort)
        }
        "refresh" => init(agent, &sort),
        "projects" => projects(),
        "follow" => {
            let f = Follow { folder: agent.project.clone(), account: text("account"), project_id: text("project"), slug: text("slug") };
            let mut all = follows();
            all.retain(|x| !(x.folder == f.folder && x.account == f.account && x.project_id == f.project_id));
            if m["on"] == true {
                all.push(f);
            }
            put("follows", &all);
            host::set_timer(10);
            init(agent, &sort)
        }
        "detail" => {
            let acc = account(&text("account"))?;
            let (issue, event) = details(&acc, &text("id"))?;
            json!({ "type": "detail", "account": acc.id, "id": text("id"), "context": sentry::context(&issue, &event) })
        }
        "propose" => {
            let job = Job { account: text("account"), id: text("id"), folder: agent.project.clone(), regressed: false };
            let p = propose(&job);
            json!({ "type": "proposal", "account": job.account, "id": job.id, "proposal": p })
        }
        "fix" => {
            draft(agent, &text("account"), &text("id"))?;
            json!({ "type": "opened" })
        }
        "status" => {
            let acc = account(&text("account"))?;
            let status = if text("status") == "ignored" { "ignored" } else { "resolved" };
            let ids: Vec<String> = m["ids"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| vec![text("id")]);
            let paths: Vec<String> = ids.iter().map(|id| format!("issues/{}/", encode(id))).collect();
            api_many(&acc, "PUT", &paths, Some(json!({ "status": status }).to_string())).into_iter().collect::<Result<Vec<_>, _>>()?;
            init(agent, &sort)
        }
        "merge" => {
            let acc = account(&text("account"))?;
            let ids: Vec<String> = m["ids"].as_array().into_iter().flatten().filter_map(|x| x.as_str().map(|id| format!("id={}", encode(id)))).collect();
            if ids.len() < 2 {
                return Err(t!("Il faut au moins deux erreurs à fusionner.", "Merging needs at least two errors."));
            }
            api(&acc, "PUT", &format!("issues/?project={}&{}", encode(&text("project")), ids.join("&")), Some(json!({ "merge": 1 }).to_string()))?;
            init(agent, &sort)
        }
        "stale" => stale(&agent.project, m["days"].as_u64().unwrap_or(30))?,
        "triage" => {
            for acc in accounts() {
                let mut ids: Vec<String> = follows().into_iter().filter(|f| f.account == acc.id).map(|f| f.project_id).collect();
                if !follows().iter().any(|f| f.folder == agent.project && f.account == acc.id) {
                    continue;
                }
                ids.sort();
                ids.dedup();
                let found = list(&acc, &ids.iter().map(String::as_str).collect::<Vec<_>>())?;
                triage(&acc, &found, true);
            }
            init(agent, &sort)
        }
        _ => return Err(format!("Unknown request {kind}.")),
    })
}

fn line(i: &Issue) -> String {
    format!("{} — {} ({} events, {} users, last seen {}, Sentry project {})", i.short_id, i.title, i.count, i.users, i.last_seen, i.project)
}

impl Extension for Sentry {
    fn on_event(name: String, agent: Option<Agent>, data: String) {
        match name.as_str() {
            "started" | "options.changed" => host::set_timer(15),
            "timer" => tick(),
            "panel.message" => {
                let data: Value = serde_json::from_str(&data).unwrap_or_default();
                let m = &data["message"];
                let Some(agent) = agent else { return };
                let kind = m["type"].as_str().unwrap_or_default();
                let mut reply = answer(kind, &agent, m).unwrap_or_else(|e| json!({ "type": "error", "for": kind, "message": e }));
                reply["to"] = m["to"].clone();
                post(reply);
            }
            _ => {}
        }
    }

    fn call_tool(name: String, input: String, agent: Option<Agent>) -> Result<String, String> {
        let agent = agent.ok_or("Call this tool from an agent started by Squadron.")?;
        let (issues, errors, _) = issues_of(&agent.project, "date");
        if issues.is_empty() && errors.is_empty() && !follows().iter().any(|f| f.folder == agent.project) {
            return Err("No Sentry project is followed for this Squadron project: ask the user to choose one in the Sentry panel.".into());
        }
        match name.as_str() {
            "issues" => {
                let lines: Vec<String> = issues.iter().filter_map(|v| serde_json::from_value::<Issue>(v["issue"].clone()).ok()).map(|i| line(&i)).collect();
                let mut out = if lines.is_empty() { "No unresolved Sentry issue in the last 14 days.".to_string() } else { lines.join("\n") };
                if !errors.is_empty() {
                    out.push_str(&format!("\n\nCould not read: {}", errors.join("; ")));
                }
                Ok(out)
            }
            "issue" => {
                let input: Value = serde_json::from_str(&input).unwrap_or_default();
                let wanted = input["id"].as_str().unwrap_or_default().trim().to_uppercase();
                let found = issues.iter().find(|v| v["issue"]["short_id"].as_str().is_some_and(|s| s.to_uppercase() == wanted) || v["issue"]["id"].as_str() == Some(wanted.as_str()));
                let found = found.ok_or("No unresolved issue of this project has this id: call sentry_issues.")?;
                let acc = account(found["account"].as_str().unwrap_or_default())?;
                let id = found["issue"]["id"].as_str().unwrap_or_default();
                let (issue, event) = details(&acc, id)?;
                let mut out = sentry::context(&issue, &event);
                if let Some(text) = found["proposal"]["text"].as_str().filter(|_| found["proposal"]["status"] == "ready") {
                    out.push_str(&format!("\n\nAn analysis written by Claude Code, unverified:\n{text}"));
                }
                Ok(out)
            }
            _ => Err(format!("Unknown tool {name}.")),
        }
    }

    fn check_account(account: Account) -> Result<String, String> {
        let host_name = field(&account.fields, "host");
        let org = field(&account.fields, "org");
        let r = host::fetch(&Request {
            method: "GET".into(),
            url: format!("https://{host_name}/api/0/organizations/{}/projects/", encode(&org)),
            headers: vec![("Authorization".into(), format!("Bearer {{{{{}.token}}}}", account.id))],
            body: None,
        })?;
        match r.status {
            200 => {
                let n = serde_json::from_str::<Value>(&r.body).ok().and_then(|v| v.as_array().map(Vec::len)).unwrap_or(0);
                Ok(json!({ "text": { "en": "Connected", "fr": "Connecté" }, "detail": { "en": format!("{n} projects"), "fr": format!("{n} projets") } }).to_string())
            }
            401 | 403 => Err(json!({ "title": { "en": "Sentry refused this token", "fr": "Sentry refuse ce jeton" }, "detail": { "en": "It may be wrong, expired, or miss the org:read and event:read scopes.", "fr": "Il est peut-être faux, expiré, ou sans les droits org:read et event:read." }, "field": "token" }).to_string()),
            404 => Err(json!({ "title": { "en": "Unknown organization", "fr": "Organisation inconnue" }, "detail": { "en": "Use the slug from your Sentry address.", "fr": "Indiquez le slug de l’adresse Sentry." }, "field": "org" }).to_string()),
            s => Err(format!("Sentry answered {s}.")),
        }
    }
}

squadron_extension!(Sentry);
