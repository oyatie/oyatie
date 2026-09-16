use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use tokio::io::AsyncWriteExt;

impl Relay {
    pub(super) async fn transaction<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: &mut BufReader<S>,
        recipient: &str,
        message: &QueuedMessage,
        hello: smtp_proto::Response<String>,
    ) -> Result<(), Failure> {
        if let Some((user, pass)) = &self.config.credentials {
            let plain = hello.message.lines().skip(1).any(|line| {
                let mut words = line.split_ascii_whitespace();
                words.next().is_some_and(|w| w.eq_ignore_ascii_case("AUTH"))
                    && words.any(|w| w.eq_ignore_ascii_case("PLAIN"))
            });
            if !plain {
                return Err(temporary().into());
            }
            let auth = STANDARD.encode(format!("\0{user}\0{pass}"));
            expect(
                command(stream, format!("AUTH PLAIN {auth}\r\n").as_bytes()).await?,
                235,
            )
            .map_err(|_| temporary())?;
        }
        let mut params = String::new();
        if !message.raw.is_ascii() {
            if !capability(&hello, "8BITMIME") {
                return Err(DeliveryOutcome::Permanent(554).into());
            }
            params.push_str(" BODY=8BITMIME");
            let header_end = message
                .raw
                .windows(4)
                .position(|v| v == b"\r\n\r\n")
                .unwrap_or(message.raw.len());
            if !message.raw[..header_end].is_ascii() {
                if !capability(&hello, "SMTPUTF8") {
                    return Err(DeliveryOutcome::Permanent(554).into());
                }
                params.push_str(" SMTPUTF8");
            }
        }
        expect(
            command(
                stream,
                format!("MAIL FROM:<{}>{params}\r\n", message.sender).as_bytes(),
            )
            .await?,
            250,
        )?;
        let reply = command(stream, format!("RCPT TO:<{recipient}>\r\n").as_bytes()).await?;
        if !matches!(reply.code, 250..=252) {
            expect(reply, 250)?;
        }
        expect(
            wire::command_for(stream, b"DATA\r\n", Duration::from_secs(120)).await?,
            354,
        )?;
        for line in message.raw.split_inclusive(|b| *b == b'\n') {
            tokio::time::timeout(Duration::from_secs(180), async {
                if line.starts_with(b".") {
                    stream
                        .get_mut()
                        .write_all(b".")
                        .await
                        .map_err(|_| temporary())?;
                }
                stream
                    .get_mut()
                    .write_all(line)
                    .await
                    .map_err(|_| temporary())?;
                Ok::<_, DeliveryOutcome>(())
            })
            .await
            .map_err(|_| temporary())??;
        }
        let reply = wire::command_for(stream, b".\r\n", Duration::from_secs(600))
            .await
            .map_err(Failure::ambiguous)?;
        let definite_refusal = (400..600).contains(&reply.code);
        expect(reply, 250).map_err(|outcome| {
            if definite_refusal {
                outcome.into()
            } else {
                Failure::ambiguous(outcome)
            }
        })?;
        // Final DATA acceptance settles delivery. QUIT/connection teardown cannot
        // reverse it and must never turn an accepted message into a retry.
        Ok(())
    }
}
