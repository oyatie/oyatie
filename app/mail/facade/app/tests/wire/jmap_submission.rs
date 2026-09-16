use super::*;

pub(super) fn verify(client: &reqwest::blocking::Client, url: &str, token: &str) {
    let call = |method: &str, arguments: Value| -> Value {
        client.post(format!("{url}/jmap")).bearer_auth(token)
            .json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],"methodCalls":[[method,arguments,"r"]]}))
            .send().unwrap().error_for_status().unwrap().json().unwrap()
    };
    let draft = call(
        "Email/set",
        json!({"accountId":"alice","create":{"draft":{
            "mailboxIds":{"inbox":true},"from":[{"email":"alice@example.org"}],
            "bcc":[{"email":"jmap@remote.org"}],"subject":"JMAP wire submission",
            "bodyStructure":{"type":"text/plain","partId":"1"},"bodyValues":{"1":{"value":"JMAP durable delivery"}}
        }}}),
    );
    let email = draft["methodResponses"][0][1]["created"]["draft"]["id"]
        .as_str()
        .expect("draft created");
    let sent = call(
        "EmailSubmission/set",
        json!({"accountId":"alice","create":{"send":{"identityId":"alice","emailId":email}},"onSuccessDestroyEmail":["#send"]}),
    );
    let id = sent["methodResponses"][0][1]["created"]["send"]["id"]
        .as_str()
        .expect("submission accepted");
    assert_eq!(sent["methodResponses"][1][0], "Email/set");
    assert_eq!(sent["methodResponses"][1][1]["destroyed"], json!([email]));
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let response = call(
            "EmailSubmission/get",
            json!({"accountId":"alice","ids":[id]}),
        );
        let record = &response["methodResponses"][0][1]["list"][0];
        assert_eq!(record["emailId"], email);
        if record["undoStatus"] == "final" {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "submission worker did not finish: {response}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let deleted = call("Email/get", json!({"accountId":"alice","ids":[email]}));
    assert_eq!(deleted["methodResponses"][0][1]["notFound"], json!([email]));
}
