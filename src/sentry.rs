use serde::{Deserialize, Serialize};
use serde_json::Value;

// What Claude reads: the stack trace and its code, not a whole event.
const MAX_CONTEXT: usize = 14_000;
const MAX_FRAMES: usize = 15;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Issue {
    pub id: String,
    pub short_id: String,
    pub title: String,
    pub culprit: String,
    pub permalink: String,
    pub level: String,
    pub substatus: String,
    pub count: String,
    pub users: u64,
    pub first_seen: String,
    pub last_seen: String,
    pub project_id: String,
    pub project: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Group {
    pub subject: String,
    pub project: String,
    pub ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Noise {
    pub id: String,
    pub reason: String,
    pub why: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Triage {
    pub ids: Vec<String>,
    pub at: u64,
    pub groups: Vec<Group>,
    pub noise: Vec<Noise>,
    pub error: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Job {
    pub account: String,
    pub id: String,
    pub folder: String,
    pub regressed: bool,
}

fn s(v: &Value) -> String {
    match v {
        Value::String(x) => x.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

pub fn issue(v: &Value) -> Issue {
    Issue {
        id: s(&v["id"]),
        short_id: s(&v["shortId"]),
        title: s(&v["title"]),
        culprit: s(&v["culprit"]),
        permalink: s(&v["permalink"]),
        level: s(&v["level"]),
        substatus: s(&v["substatus"]),
        count: s(&v["count"]),
        users: v["userCount"].as_u64().unwrap_or(0),
        first_seen: s(&v["firstSeen"]),
        last_seen: s(&v["lastSeen"]),
        project_id: s(&v["project"]["id"]),
        project: s(&v["project"]["slug"]),
    }
}

pub fn issues(body: &str) -> Vec<Issue> {
    serde_json::from_str::<Value>(body).ok().and_then(|v| v.as_array().map(|a| a.iter().map(issue).collect())).unwrap_or_default()
}

pub struct Fresh {
    pub found: Vec<(Issue, bool)>,
    pub seen: Vec<String>,
    pub regressed: Vec<String>,
}

// New issues, and known ones that came back; a project seen for the first time only fills `seen`.
pub fn fresh(list: &[Issue], seen: &[String], regressed: &[String], primed: impl Fn(&str) -> bool) -> Fresh {
    let mut out = Fresh { found: Vec::new(), seen: seen.to_vec(), regressed: regressed.to_vec() };
    for i in list {
        let known = seen.contains(&i.id);
        if !known {
            out.seen.push(i.id.clone());
            if primed(&i.project_id) {
                out.found.push((i.clone(), false));
            }
        } else if i.substatus == "regressed" && !regressed.contains(&i.id) {
            out.found.push((i.clone(), true));
        }
        if i.substatus == "regressed" {
            if !out.regressed.contains(&i.id) {
                out.regressed.push(i.id.clone());
            }
        } else {
            out.regressed.retain(|x| x != &i.id);
        }
    }
    let over = out.seen.len().saturating_sub(3000);
    out.seen.drain(..over);
    out
}

fn frame_line(f: &Value) -> String {
    let file = [&f["filename"], &f["absPath"], &f["module"]].iter().map(|v| s(v)).find(|x| !x.is_empty()).unwrap_or_else(|| "?".into());
    let line = f["lineNo"].as_u64().map(|n| format!(":{n}")).unwrap_or_default();
    let function = s(&f["function"]);
    let mut out = if function.is_empty() { format!("  {file}{line}") } else { format!("  {file}{line} in {function}") };
    let at = f["lineNo"].as_u64();
    for c in f["context"].as_array().into_iter().flatten() {
        let n = c[0].as_u64();
        let mark = if n.is_some() && n == at { ">" } else { " " };
        out.push_str(&format!("\n   {mark} {:>5} | {}", n.map(|x| x.to_string()).unwrap_or_default(), s(&c[1])));
    }
    out
}

fn tags(event: &Value, keys: &[&str]) -> Vec<String> {
    event["tags"].as_array().into_iter().flatten().filter(|t| keys.contains(&s(&t["key"]).as_str())).map(|t| format!("{}={}", s(&t["key"]), s(&t["value"]))).collect()
}

fn cut(text: &str, max: usize) -> String {
    text.trim().chars().take(max).collect()
}

// What the triage reads of an event: where it ran and which release.
pub fn facts(event: &Value) -> String {
    tags(event, &["environment", "release", "server_name", "url"]).iter().map(|t| cut(t, 120)).collect::<Vec<_>>().join(" ")
}

pub fn brief(i: &Issue, facts: &str) -> String {
    format!("{} | {} | {} | {} | {} | {} | {} events, {} users | first {} | last {}", i.short_id, i.project, i.level, cut(&i.title, 200), cut(&i.culprit, 120), facts, i.count, i.users, i.first_seen, i.last_seen)
}

// The model answers with short ids; only listed issues are kept, and a group stays within one project.
pub fn read_triage(answer: &str, list: &[Issue]) -> Option<(Vec<Group>, Vec<Noise>)> {
    let v: Value = serde_json::from_str(answer.get(answer.find('{')?..=answer.rfind('}')?)?).ok()?;
    let find = |x: &Value| list.iter().find(|i| i.short_id.eq_ignore_ascii_case(s(x).trim()));
    let mut groups = Vec::new();
    for g in v["groups"].as_array().into_iter().flatten() {
        let mut members: Vec<&Issue> = Vec::new();
        for i in g["ids"].as_array().into_iter().flatten().filter_map(find) {
            if !members.iter().any(|m| m.id == i.id) {
                members.push(i);
            }
        }
        if members.len() >= 2 && members.iter().all(|m| m.project_id == members[0].project_id) {
            groups.push(Group { subject: cut(&s(&g["subject"]), 120), project: members[0].project_id.clone(), ids: members.iter().map(|m| m.id.clone()).collect() });
        }
    }
    let mut noise: Vec<Noise> = Vec::new();
    for n in v["noise"].as_array().into_iter().flatten() {
        let Some(i) = find(&n["id"]) else { continue };
        if noise.iter().any(|x| x.id == i.id) {
            continue;
        }
        let reason = s(&n["reason"]);
        let reason = if ["mep", "local", "test"].contains(&reason.as_str()) { reason } else { "other".into() };
        noise.push(Noise { id: i.id.clone(), reason, why: cut(&s(&n["why"]), 200) });
    }
    Some((groups, noise))
}

pub fn triage_system(fr: bool) -> &'static str {
    if fr {
        "Tu tries des erreurs Sentry pour une équipe de développeurs. Tu n’as aucun outil. Réponds uniquement par un objet JSON valide, sans texte autour ni bloc de code."
    } else {
        "You sort Sentry errors for a team of developers. You have no tools. Answer only with a valid JSON object, with no text around it and no code block."
    }
}

pub fn triage_prompt(fr: bool, now: &str) -> String {
    if fr {
        format!(
            "Nous sommes le {now} (UTC). Voici les erreurs Sentry non résolues, une par ligne : identifiant | projet | niveau | titre | origine | tags du dernier événement | volume | première et dernière occurrence.\n\n\
1. Regroupe les erreurs d’un même projet qui ont visiblement la même cause : même exception au même endroit, même service tiers en panne, même requête. Un groupe a au moins deux erreurs. N’invente pas de groupe.\n\
2. Repère les erreurs qui n’ont aucune raison d’être traitées :\n\
- \"mep\" : une erreur passagère d’une mise en production (cache ou conteneur pas prêt, migration pas encore jouée, classe ou fichier introuvable, maintenance), apparue sur une courte période et plus revue depuis ;\n\
- \"local\" : un environnement local ou de développement (environment local, dev ou development, localhost, 127.0.0.1, .lndo.site, .test, .local, .ddev.site) ;\n\
- \"test\" : une erreur déclenchée volontairement ou par un test (message de test, route de test, robot de recette) ;\n\
- \"other\" : une autre raison évidente (robot ou scan sur des adresses qui n’existent pas, navigateur obsolète, extension de navigateur).\n\
Dans le doute, ne la mets pas.\n\n\
Réponds avec exactement cet objet, les textes en français :\n\
{{\"groups\":[{{\"subject\":\"le sujet commun en quelques mots\",\"ids\":[\"SHOP-1A\",\"SHOP-2B\"]}}],\"noise\":[{{\"id\":\"SHOP-3C\",\"reason\":\"mep\",\"why\":\"la raison en une phrase courte\"}}]}}\n\n\
Erreurs :"
        )
    } else {
        format!(
            "It is {now} (UTC). Here are the unresolved Sentry errors, one per line: id | project | level | title | culprit | tags of the latest event | volume | first and last seen.\n\n\
1. Group the errors of one project that clearly share a cause: same exception at the same place, same third-party service down, same query. A group has at least two errors. Do not make up groups.\n\
2. Find the errors that have no reason to be worked on:\n\
- \"mep\": a passing error of a deployment (cache or container not ready, migration not run yet, class or file not found, maintenance), seen over a short time and not since;\n\
- \"local\": a local or development environment (environment local, dev or development, localhost, 127.0.0.1, .lndo.site, .test, .local, .ddev.site);\n\
- \"test\": an error raised on purpose or by a test (test message, test route, QA robot);\n\
- \"other\": another obvious reason (robot or scan on addresses that do not exist, outdated browser, browser extension).\n\
When in doubt, leave it out.\n\n\
Answer with exactly this object, the texts in English:\n\
{{\"groups\":[{{\"subject\":\"the common subject in a few words\",\"ids\":[\"SHOP-1A\",\"SHOP-2B\"]}}],\"noise\":[{{\"id\":\"SHOP-3C\",\"reason\":\"mep\",\"why\":\"the reason in one short sentence\"}}]}}\n\n\
Errors:"
        )
    }
}

// Howard Hinnant's days-to-civil: the extension has no date library.
pub fn iso(ms: u64) -> String {
    let secs = ms / 1000;
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}Z", secs % 86_400 / 3600, secs % 3600 / 60)
}

pub fn context(issue: &Value, event: &Value) -> String {
    let mut out = vec![format!("Sentry issue {}: {}", s(&issue["shortId"]), s(&issue["title"]))];
    let culprit = s(&issue["culprit"]);
    if !culprit.is_empty() {
        out.push(format!("Culprit: {culprit}"));
    }
    out.push(format!(
        "Level: {} · Events: {} · Users: {} · First seen: {} · Last seen: {}",
        s(&issue["level"]),
        s(&issue["count"]),
        issue["userCount"].as_u64().unwrap_or(0),
        s(&issue["firstSeen"]),
        s(&issue["lastSeen"])
    ));
    let tags = tags(event, &["environment", "release", "url", "transaction", "server_name", "browser", "os"]);
    if !tags.is_empty() {
        out.push(format!("Tags: {}", tags.join(" · ")));
    }
    for entry in event["entries"].as_array().into_iter().flatten() {
        let data = &entry["data"];
        match s(&entry["type"]).as_str() {
            "request" => out.push(format!("Request: {} {}", s(&data["method"]), s(&data["url"]))),
            "message" => out.push(format!("Message: {}", [s(&data["formatted"]), s(&data["message"])].into_iter().find(|m| !m.is_empty()).unwrap_or_default())),
            "exception" => {
                for e in data["values"].as_array().into_iter().flatten().rev() {
                    out.push(format!("Exception: {}: {}", s(&e["type"]), s(&e["value"])));
                    let frames: Vec<&Value> = e["stacktrace"]["frames"].as_array().map(|f| f.iter().rev().collect()).unwrap_or_default();
                    let in_app: Vec<&Value> = frames.iter().copied().filter(|f| f["inApp"] == true).collect();
                    let shown = if in_app.is_empty() { frames } else { in_app };
                    out.push("Stack trace, most recent call first:".into());
                    out.extend(shown.iter().take(MAX_FRAMES).map(|f| frame_line(f)));
                }
            }
            _ => {}
        }
    }
    let text = out.join("\n");
    if text.len() <= MAX_CONTEXT {
        return text;
    }
    let mut end = MAX_CONTEXT;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[…]", &text[..end])
}

pub fn prompt(fr: bool) -> &'static str {
    if fr {
        "L’extension Sentry de Squadron t’appelle pour analyser une erreur de production. Son détail et sa stack trace sont dans l’entrée standard. Le dossier courant est le dépôt du projet : lis le code nécessaire, sans rien modifier. Réponds en français, sans préambule ni conclusion, avec exactement ces trois sections :\n## Diagnostic\n2 à 4 phrases : ce qui se passe et la cause la plus probable, avec les fichiers et les lignes.\n## Plan d’exécution\nLes étapes numérotées de la correction : les fichiers et fonctions à modifier, ce qu’il faut y changer, le test à ajouter. Si la correction tient en quelques lignes, ajoute un diff unifié minimal dans un bloc ```diff. Si aucune correction du code n’est sûre, dis ce qu’il faut vérifier d’abord.\n## À vérifier\n1 à 3 points : les risques de la correction et comment la tester."
    } else {
        "Squadron's Sentry extension calls you to analyse a production error. Its details and stack trace are on standard input. The current folder is the project's repository: read the code you need, without changing anything. Answer in English, with no preamble or conclusion, with exactly these three sections:\n## Diagnosis\n2 to 4 sentences: what happens and the most likely cause, with files and lines.\n## Plan\nThe numbered steps of the fix: the files and functions to change, what to change in them, the test to add. When the fix is a few lines, add a minimal unified diff in a ```diff block. When no code fix is safe, say what to check first.\n## To check\n1 to 3 points: the risks of the fix and how to test it."
    }
}

// Over the user's own Claude Code settings: this run only writes a proposal that someone reads.
pub fn system(fr: bool) -> &'static str {
    if fr {
        "Cette session sert seulement à rédiger une proposition qu’une personne relira. Tu n’as que des outils de lecture : n’essaie jamais de modifier un fichier ni de lancer une commande, et ne propose pas de le faire. Suis exactement le format demandé, avec ses trois titres, sans rien avant ni après."
    } else {
        "This session only writes a proposal that a person will read. You only have reading tools: never try to change a file or run a command, and do not offer to. Follow the requested format exactly, with its three headings, with nothing before or after."
    }
}

pub fn enqueue(queue: &mut Vec<Job>, job: Job) {
    if !queue.iter().any(|j| j.account == job.account && j.id == job.id) {
        queue.push(job);
    }
    let over = queue.len().saturating_sub(20);
    queue.drain(..over);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn listed(id: &str, project: &str, substatus: &str) -> Issue {
        Issue { id: id.into(), project_id: project.into(), substatus: substatus.into(), ..Default::default() }
    }

    #[test]
    fn reads_the_issue_list() {
        let body = r#"[{"id":"42","shortId":"SHOP-1A","title":"TypeError: x","culprit":"App\\Foo in bar","permalink":"https://acme.sentry.io/issues/42/","level":"error","substatus":"new","count":"12","userCount":3,"firstSeen":"2026-11-24T10:00:00Z","lastSeen":"2026-11-24T11:00:00Z","project":{"id":"7","slug":"shop"}}]"#;
        let i = &issues(body)[0];
        assert_eq!((i.id.as_str(), i.short_id.as_str(), i.count.as_str(), i.users, i.project.as_str()), ("42", "SHOP-1A", "12", 3, "shop"));
        assert!(issues("{\"detail\":\"no\"}").is_empty());
    }

    #[test]
    fn first_look_only_remembers() {
        let f = fresh(&[listed("1", "7", "new")], &[], &[], |_| false);
        assert!(f.found.is_empty());
        assert_eq!(f.seen, ["1"]);
    }

    #[test]
    fn new_and_regressed_issues_come_once() {
        let list = [listed("1", "7", "ongoing"), listed("2", "7", "new"), listed("3", "7", "regressed")];
        let f = fresh(&list, &["1".into(), "3".into()], &[], |_| true);
        assert_eq!(f.found.iter().map(|(i, r)| (i.id.as_str(), *r)).collect::<Vec<_>>(), [("2", false), ("3", true)]);
        let again = fresh(&list, &f.seen, &f.regressed, |_| true);
        assert!(again.found.is_empty());
        let resolved = fresh(&[listed("3", "7", "ongoing")], &again.seen, &again.regressed, |_| true);
        assert!(resolved.regressed.is_empty());
    }

    #[test]
    fn context_keeps_in_app_frames_and_code() {
        let issue = json!({"shortId":"SHOP-1A","title":"Undefined variable","culprit":"App\\Cart::total","level":"error","count":"3","userCount":1});
        let event = json!({"tags":[{"key":"environment","value":"prod"},{"key":"user","value":"x"}],"entries":[
            {"type":"request","data":{"method":"GET","url":"https://shop.test/cart"}},
            {"type":"exception","data":{"values":[{"type":"ErrorException","value":"Undefined variable $total","stacktrace":{"frames":[
                {"filename":"vendor/symfony/HttpKernel.php","lineNo":10,"inApp":false},
                {"filename":"src/Cart.php","lineNo":42,"function":"total","inApp":true,"context":[[41,"    $sum = 0;"],[42,"    return $total;"]]}
            ]}}]}}]});
        let text = context(&issue, &event);
        assert!(text.contains("Tags: environment=prod"));
        assert!(!text.contains("user=x"));
        assert!(text.contains("Request: GET https://shop.test/cart"));
        assert!(text.contains("  src/Cart.php:42 in total\n        41 |     $sum = 0;\n   >    42 |     return $total;"));
        assert!(!text.contains("HttpKernel"));
    }

    #[test]
    fn long_context_is_cut_on_a_character() {
        let issue = json!({"shortId":"A","title":"é".repeat(MAX_CONTEXT)});
        assert!(context(&issue, &json!({})).ends_with("[…]"));
    }

    #[test]
    fn triage_keeps_known_ids_and_one_project_groups() {
        let mut a = listed("1", "7", "new");
        a.short_id = "SHOP-1".into();
        let mut b = listed("2", "7", "new");
        b.short_id = "SHOP-2".into();
        let mut c = listed("3", "8", "new");
        c.short_id = "API-3".into();
        let answer = r#"Voici :
{"groups":[{"subject":"Redis","ids":["shop-1","SHOP-2","SHOP-1","NOPE-9"]},{"subject":"cross","ids":["SHOP-1","API-3"]},{"subject":"alone","ids":["API-3"]}],
 "noise":[{"id":"API-3","reason":"local","why":"localhost"},{"id":"API-3","reason":"mep","why":"twice"},{"id":"SHOP-2","reason":"weird","why":"x"},{"id":"GONE-1","reason":"mep","why":"y"}]}
fin"#;
        let (groups, noise) = read_triage(answer, &[a, b, c]).unwrap();
        assert_eq!(groups, [Group { subject: "Redis".into(), project: "7".into(), ids: vec!["1".into(), "2".into()] }]);
        assert_eq!(noise.iter().map(|n| (n.id.as_str(), n.reason.as_str())).collect::<Vec<_>>(), [("3", "local"), ("2", "other")]);
        assert!(read_triage("pas de JSON", &[]).is_none());
    }

    #[test]
    fn brief_has_the_event_facts() {
        let event = json!({"tags":[{"key":"environment","value":"local"},{"key":"url","value":"http://shop.lndo.site/"},{"key":"user","value":"x"}]});
        let f = facts(&event);
        assert_eq!(f, "environment=local url=http://shop.lndo.site/");
        let i = Issue { short_id: "SHOP-1".into(), project: "shop".into(), title: "Boom".into(), count: "3".into(), ..Default::default() };
        assert!(brief(&i, &f).starts_with("SHOP-1 | shop |  | Boom |  | environment=local"));
    }

    #[test]
    fn iso_dates() {
        assert_eq!(iso(0), "1970-01-01T00:00Z");
        assert_eq!(iso(1_700_000_000_000), "2023-11-14T22:13Z");
        assert_eq!(iso(1_795_800_000_000), "2026-11-27T17:20Z");
    }

    #[test]
    fn queue_has_no_repeats_and_a_cap() {
        let mut q = Vec::new();
        for n in 0..25 {
            enqueue(&mut q, Job { account: "a".into(), id: n.to_string(), folder: "/p".into(), regressed: false });
        }
        enqueue(&mut q, Job { account: "a".into(), id: "24".into(), folder: "/p".into(), regressed: false });
        assert_eq!(q.len(), 20);
        assert_eq!(q[0].id, "5");
    }
}
