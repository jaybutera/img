// Img server address
//export const img_server: string = "http://localhost:2342";
export const img_server: string = "https://img.smdhi.xyz:8080";

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
