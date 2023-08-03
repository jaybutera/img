import { get_image_names } from '$lib/img.ts';

export async function load({ params }) {
    const imgs = await get_image_names(params.topic);
    return {
        topic: params.topic,
        images: imgs,
    }
}
