import { get_challenge } from '$lib/img.ts';

export function load({ params }) {
    let challenge = await get_challenge();
    return {
        challenge: challenge,
    }
}
