import { get_image_names, get_index } from '$lib/img.ts';

export function load({ params }) {
    const imgs = get_image_names(params.index);
    return {
        index_name: params.index,
        topics: get_index(params.index).topics,
    }
}
