// Img server address
const img_server: string = "http://localhost:5173";

async function get_image_names(): Promise<string[]> {
  try {
    // Make a GET request to the endpoint
    const response = await fetch(f'{img_server}/{topic}/images');

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

