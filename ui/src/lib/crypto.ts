import { sign } from 'tweetnacl-ts';

export async function sign(
    key_bytes: Uint8Array,
    message: Uint8Array,
): Promise<string> {
    let keypair = box.keyPair.fromSecretKey(key_bytes);
    let sig = sign(message, keypair.secretKey);
    return sig;
}
