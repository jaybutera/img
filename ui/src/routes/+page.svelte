<script>
    import 'bootstrap/dist/css/bootstrap.min.css';
    import { handle_file_upload } from "$lib/img.ts";
    import Nav from '../components/Nav.svelte';
    import Uploading from '../components/Uploading.svelte';
    import { goto } from "$app/navigation";
    export let data;
    let not_uploading = true;
    let selected_files;
    let topic;

    async function upload_file(event) {
        // Get the file element
        let task = handle_file_upload(topic, selected_files);
        not_uploading = false;
        await task;
        goto(`/${topic}`);
    }
</script>

<style>
    body {
        height: 100%;
    }
    .new-form {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-top: 100px;
    }
    .nt-form {
        width: 30%;
        height: 50px;
        font-size: 20px;
        border-radius: 10px;
        border: 1px solid #ccc;
        padding: 10px;
    }
    .new-topic {
        margin: 30px;
    }
    .input-images {
        margin: 10px;
    }
    .submit-topic {
        margin: 30px;
        background-color: #ccc;
    }
    h1 {
        margin-top: 100px;
        text-align: center;
    }
</style>

<Nav />
{#if not_uploading}
<h1>Start a New Collection!</h1>
<div class="new-form">
        <input bind:value={topic} type="text" class="nt-form new-topic" placeholder="topic name">
        <input bind:files={selected_files} type="file" multiple class="nt-form input-images">
        <button on:click={upload_file}
            class="nt-form submit-topic">Submit</button>
</div>
{:else}
<Uploading />
{/if}
