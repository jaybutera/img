import { get_image_names } from '$lib/img.ts';

export function load({ params }) {
    const imgs = get_image_names(params.topic);
    return {
        topic: params.topic,
        images: imgs,
    }
}
