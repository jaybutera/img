import { get_all_indexes } from '$lib/img.ts';
export async function load({ params }) {
    const indexes = await get_all_indexes();
    console.log("indexes: " + JSON.stringify(indexes));
    return {
        indexes: indexes,
    }
}
