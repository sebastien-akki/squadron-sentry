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
    let tags: Vec<String> = event["tags"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|t| ["environment", "release", "url", "transaction", "server_name", "browser", "os"].contains(&s(&t["key"]).as_str()))
        .map(|t| format!("{}={}", s(&t["key"]), s(&t["value"])))
        .collect();
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
        "L’extension Sentry de Squadron t’appelle pour une erreur de production qui vient d’arriver. Son détail et sa stack trace sont dans l’entrée standard. Le dossier courant est le dépôt du projet : lis le code nécessaire, sans rien modifier. Réponds en français, sans préambule ni conclusion, avec exactement ces trois sections :\n## Diagnostic\n2 à 4 phrases : ce qui se passe et la cause la plus probable, avec les fichiers et les lignes.\n## Correction proposée\nUn diff unifié minimal dans un bloc ```diff, ou, si aucune correction du code n’est sûre, ce qu’il faut vérifier.\n## À vérifier\n1 à 3 points : les risques de la correction et comment la tester."
    } else {
        "Squadron's Sentry extension calls you for a production error that just came in. Its details and stack trace are on standard input. The current folder is the project's repository: read the code you need, without changing anything. Answer in English, with no preamble or conclusion, with exactly these three sections:\n## Diagnosis\n2 to 4 sentences: what happens and the most likely cause, with files and lines.\n## Proposed fix\nA minimal unified diff in a ```diff block, or, when no code fix is safe, what to check.\n## To check\n1 to 3 points: the risks of the fix and how to test it."
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
