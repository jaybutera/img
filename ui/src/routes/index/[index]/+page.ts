import { get_index } from '$lib/img.ts';

export async function load({ params }) {
    const index = await get_index(params.index);
    return {
        index_name: params.index,
        topics: index.topics,
    }
}
