<script>
    import Nav from "../../components/Nav.svelte";
    import { img_server, handle_file_upload } from "$lib/img.ts";
    import { goto } from "$app/navigation";
    import Uploading from '../../components/Uploading.svelte';
    export let data;
    const imgs = data.images;
    const topic = data.topic;
    let not_uploading = true;
    let selected_files;

    async function upload_file(event) {
        let task = handle_file_upload(topic, selected_files);
        not_uploading = false;
        await task;
        not_uploading = true;
        window.location.reload();
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
    <!--<a href="/new">Add Photos</a>-->
    <!-- Hidden input for file upload -->
    <input bind:files={selected_files} type='file' id='file' on:change={upload_file} multiple style='display: none;' />
    <!-- Visible button that triggers the hidden input -->
    <button on:click={() => document.getElementById('file').click()}>Upload File</button>
</Nav>

{#if not_uploading}
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
{:else}
    <Uploading />
{/if}
