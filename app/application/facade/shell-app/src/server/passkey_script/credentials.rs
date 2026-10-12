pub(super) const SCRIPT: &str = r#"
function encodeCredentialBytes(value) {
  return btoa(String.fromCharCode(...new Uint8Array(value)))
    .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
}
function decodeCredentialBytes(value) {
  if (typeof value !== 'string' || !/^[A-Za-z0-9_-]+$/.test(value)) throw new Error('Invalid passkey options.');
  const bytes = Uint8Array.from(atob(value.replace(/-/g, '+').replace(/_/g, '/')), ch => ch.charCodeAt(0));
  if (encodeCredentialBytes(bytes) !== value) throw new Error('Invalid passkey options.');
  return bytes;
}
function browserPublicKey(options, registration) {
  if (!options || options.challenge !== options.publicKey?.challenge) throw new Error('Invalid passkey options.');
  const key = structuredClone(options.publicKey);
  key.challenge = decodeCredentialBytes(key.challenge);
  if (registration) key.user.id = decodeCredentialBytes(key.user.id);
  for (const field of ['allowCredentials', 'excludeCredentials']) {
    if (key[field]) key[field] = key[field].map(item => ({ ...item, id: decodeCredentialBytes(item.id) }));
  }
  return key;
}
function serializeCredential(credential, registration) {
  if (!credential || credential.type !== 'public-key') throw new Error('Passkey unavailable. Retry.');
  const response = { clientDataJSON: encodeCredentialBytes(credential.response.clientDataJSON) };
  if (registration) {
    response.attestationObject = encodeCredentialBytes(credential.response.attestationObject);
    response.transports = credential.response.getTransports?.() || [];
  } else {
    response.authenticatorData = encodeCredentialBytes(credential.response.authenticatorData);
    response.signature = encodeCredentialBytes(credential.response.signature);
    response.userHandle = credential.response.userHandle ? encodeCredentialBytes(credential.response.userHandle) : null;
  }
  return { id: credential.id, rawId: encodeCredentialBytes(credential.rawId), type: credential.type, response };
}
"#;
