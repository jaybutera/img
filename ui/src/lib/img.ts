// Img server address
//export const img_server: string = "http://127.0.0.1:2342";
export const img_server: string = "https://img.smdhi.xyz:8080";

interface Index {
    name: string;
    topics: string[];
}

export async function get_image_names(topic: string): Promise<string[]> {
  try {
    // Make a GET request to the endpoint
    const response = await fetch(`${img_server}/${topic}/images`);

    // If the request was successful, parse the response as JSON
    if (response.ok) {
      const data: string[] = await response.json();
      return data;
    } else {
      console.error('Error:', response.status, response.statusText);
      return [];
    }
  } catch (error) {
    console.error('Error:', error);
    return [];
  }
}

export async function handle_file_upload(topic: string, files) {
    // Wait for all files to upload
    let filePromises = Array.from(files)
        .map((file) => processFile(topic, file));
    await Promise.all(filePromises);
}

async function processFile(topic: string, file) {
    let response = await fetch(`${img_server}/${topic}/new-image`, {
      method: 'POST',
      body: file,
    });
    
    if (!response.ok) {
      throw new Error(`Error in upload: ${response.statusText}`);
    }
}

export async function get_all_indexes(): Promise<list[string]> {
    const response = await fetch(`${img_server}/all-indexes`);

    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.json() || [];
}
export async function get_index(index: string): Promise<Index> {
    const response = await fetch(`${img_server}/index/${index}`);

    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }

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
