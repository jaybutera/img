<script>
    import 'bootstrap/dist/css/bootstrap.min.css';
    import { handle_file_upload } from "$lib/img.ts";
    import Nav from '../../components/Nav.svelte';
    import Uploading from '../../components/Uploading.svelte';
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
    h1 {
        margin-top: 2em;
        text-align: center;
    }
</style>

<Nav />
{#if not_uploading}
<h1>Start a New Collection!</h1>
<div class="vert-new-form">
    <div class="new-form">
            <input bind:value={topic} type="text" class="nt-field nt-form new-form" placeholder="topic name">
            <input bind:files={selected_files} type="file" multiple class="nt-form input-images">
            <button on:click={upload_file}
                class="nt-form submit-form">Submit</button>
    </div>
</div>
{:else}
<Uploading />
{/if}
