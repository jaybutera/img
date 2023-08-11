import { get_image_names, get_index, img_server } from '$lib/img.ts';

export async function load({ params }) {
    const index = await get_index(params.index);

    // for each index topic, get_image_names and take the first
    const images = await Promise.all(index.topics.map(async (topic) => {
        const image_names = await get_image_names(topic);
        let name = image_names[0];
        // if name ends in .mp4 iterate until otherwise
        while (name.endsWith('.mp4')) {
            name = image_names.shift();
        }
        return name;
    }));

    return {
        index_name: params.index,
        topics: index.topics,
        image_names: images,
    }
}
