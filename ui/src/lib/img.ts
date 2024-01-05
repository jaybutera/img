import { sign } from 'tweetnacl';
import { Buffer } from 'buffer';
import * as ed from '@noble/ed25519';
//import * as nacl from 'tweetnacl';
// Img server address
export const img_server: string = "http://127.0.0.1:2342";
//export const img_server: string = "https://img.smdhi.xyz:8080";

interface Index {
    name: string;
    topics: string[];
}

export async function authenticate(challenge: Uint8Array): Promise<void> { 
    let private_key = localStorage.getItem('private_key');
    const decoded = Buffer.from(private_key, 'base64');
    // Convert private key to Uint8Array
    //let keypair = sign.keyPair.fromSecretKey(decoded);
    const pubKey = await ed.getPublicKeyAsync(decoded);
    const sig = await ed.signAsync(challenge, decoded);
    // pubkey to base64 string
    const pubKeyStr = Buffer.from(pubKey).toString('base64');
    /*
    let sig = sign(challenge, keypair.secretKey);
        console.log(JSON.stringify({
            signature: [...sig],
            public_key: [...keypair.publicKey],
        }));
        */
    console.log(JSON.stringify({
            signature: [...sig],
            public_key: pubKeyStr,
        }));

    const response = await fetch(`${img_server}/authenticate`, {
        method: 'POST',
        credentials: 'include',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            signature: [...sig],
            public_key: pubKeyStr,
        }),
    });

    throw_err(response, "Error authenticating");
}

export async function get_challenge(): Promise<Uint8Array> {
    const response = await fetch(`${img_server}/generate-challenge`, {
        credentials: 'include',
    });
    throw_err(response, "Error getting challenge");
    let encoded = await response.json();
    const decoded = Buffer.from(encoded, 'base64');
    return decoded;
}

export async function generate_key(): Promise<Uint8Array> {
    let response = await fetch(`${img_server}/generate-key`);
    throw_err(response, "Error generating key");
    return response.text();
}

export async function rm_tag(topic: string, tag: string): Promise<string> {
    let response = await fetch(`${img_server}/${topic}/remove-tag`, {
        method: 'POST',
        body: `"${tag}"`,
    });
    throw_err(response, "Error removing tag");

    return response;
}

export async function add_tag(topic: string, tag: string): Promise<string> {
    let response = await fetch(`${img_server}/${topic}/new-tag`, {
        method: 'POST',
        body: `"${tag}"`,
    });
    throw_err(response, "Error adding tag");
    
    return response;
}

export async function get_tags(topic: string): Promise<string[]> {
    let response = await fetch(`${img_server}/${topic}/tags`);
    throw_err(response, "Error retrieving tags");

    return await response.json() || [];
}

export async function get_image_names(topic: string): Promise<string[]> {
  try {
    const pubkey = await get_pubkey();
    const pkid = await pubkey_id(pubkey);

    // Make a GET request to the endpoint
    const response = await fetch(`${img_server}/${pkid}/${topic}/images`);

    // If the request was successful, parse the response as JSON
    if (response.ok) {
      const data: string[] = await response.json();
      return data;
    } else {
      const text = await response.text();
      console.error('Error:', response.status, text);
      return [];
    }
  } catch (error) {
    console.error('Error:', error);
    return [];
  }
}

export async function handle_file_upload(topic: string, files) {
    const pubkey = await get_pubkey();
    const pkid = await pubkey_id(pubkey);
    /*
    // Wait for all files to upload
    let filePromises = Array.from(files)
        .map((file) => processFile(topic, file));
    await Promise.all(filePromises);
    */
    // Multipart form upload as a post
    let formData = new FormData();
    for (let file of files) {
        formData.append(`${file.name}`, file);
    }
    // View files in formdata
    for (let pair of formData.entries()) {
        console.log(pair[0]+ ', ' + pair[1]);
    }
    let response = await fetch(`${img_server}/${pkid}/${topic}/new-image`, {
        method: 'POST',
        credentials: 'include',
        body: formData,
    });

    throw_err(response, "Error in upload");
}

export async function get_pubkey(): string {
    const sk = localStorage.getItem('private_key');
    const decoded = Buffer.from(sk, 'base64');
    const pubKey = await ed.getPublicKeyAsync(decoded);
    const pubKeyStr = Buffer.from(pubKey).toString('base64');
    return pubKeyStr;
}

async function pubkey_id(pk: string): string {
    const decoded = Buffer.from(pk, 'base64');
    const pubKey = await ed.getPublicKeyAsync(decoded);
    const pubKeyStr = Buffer.from(pubKey).toString('base64');
    return pubKeyStr.slice(0, 8);
}

/*
async function processFile(topic: string, file) {
    let response = await fetch(`${img_server}/${topic}/new-image`, {
        method: 'POST',
        credentials: 'include',
        body: file,
    });
    
    if (!response.ok) {
      throw new Error(`Error in upload: ${response.statusText}`);
    }
}
*/

export async function get_all_indexes(): Promise<list[string]> {
    const response = await fetch(`${img_server}/all-indexes`);
    throw_err(response, "Error getting indexes");

    return await response.json() || [];
}
export async function get_index(index: string): Promise<Index> {
    const response = await fetch(`${img_server}/index/${index}`);
    throw_err(response, "Error getting index");

    // Parse the json and check it matches the `Index` type
    const data: Index = await response.json();

    return data;
}

export async function create_index(
    index: string,
    topics: list[string]): Promise<void> {
    // Make a POST request to the img server /new-index with a json body
    const response = await fetch(`${img_server}/new-index`, {
        method: 'POST',
        body: JSON.stringify({
            name: index,
            topics: topics
        }),
    });
}

async function throw_err(response: Response, msg: string) {
    if (!response.ok) {
        const err_str = await response.text();
        throw new Error(`${msg}: ${response.status} ${err_str}`);
    }
}
