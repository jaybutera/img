import { get_image_names } from '$lib/img.ts';

export function load({ params }) {
    const imgs = get_image_names('malta-2022');
    return {
        images: imgs,
    }
}
