<script>
    import Nav from "../../components/Nav.svelte";
    import { img_server } from "$lib/img.ts";
    import { goto } from "$app/navigation";
    export let data;
    const imgs = data.images;
    const topic = data.topic;

    async function handleFileUpload(event) {
        let files = event.target.files;
      
        // Wait for all files to upload
        let filePromises = Array.from(files).map(processFile);
        await Promise.all(filePromises);

        goto(`/${topic}`);
    }

    async function processFile(file) {
        let response = await fetch(`${img_server}/${topic}/new-image`, {
          method: 'POST',
          body: file,
        });
        
        if (!response.ok) {
          throw new Error(`Error in upload: ${response.statusText}`);
        }
    }
</script>

<style>
    body {
        margin: 0;
        padding: 0;
    }
    .grid {
        display: flex;
        justify-content: space-evenly;
        flex-wrap: wrap;
        align-items: center;
    }
    .media-container {
        max-width: 33%;
    }
    .media {
        max-width: 100%;
    }
    button {
    background-color: grey;
    border: none;
    color: white;
    padding: 15px 32px;
    text-align: center;
    text-decoration: none;
    display: inline-block;
    font-size: 16px;
  }
</style>

<Nav>
    <a href="/new">Add Photos</a>
    <!-- Hidden input for file upload -->
    <input type='file' id='file' on:change={handleFileUpload} multiple style='display: none;' />
    <!-- Visible button that triggers the hidden input -->
    <button on:click={() => document.getElementById('file').click()}>Upload File</button>
</Nav>

<div class="grid">
    {#each imgs as name (name)}
        {#if name.endsWith(".mp4")}
            <div class="media-container">
                <video class="media" controls>
                    <source src="{img_server}/img/{ name }" type="video/mp4">
                </video>
            </div>
        {:else}
        <div class="media-container">
            <a href="{ img_server }/img/{ name }">
                <img class="media" src="{img_server}/thumbnail/{ name }"/>
            </a>
        </div>
        {/if}
    {/each}
</div>
