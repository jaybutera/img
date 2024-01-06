<script>
    import Nav from "../../components/Nav.svelte";
    import TopicSettings from "../../components/TopicSettings.svelte";
    import { get_image_names, img_server, handle_file_upload } from "$lib/img.ts";
    import { goto } from "$app/navigation";
    import Uploading from '../../components/Uploading.svelte';
    import Modal from '../../components/Modal.svelte';
    import ErrorMessage from '../../components/ErrorMessage.svelte';
    import { createEventDispatcher } from 'svelte';
    import { onMount } from 'svelte';
    const dispatch = createEventDispatcher();

    export let data;
    //const imgs = data.images;
    let imgs = [];
    const topic = data.topic;
    let not_uploading = true;
    let selected_files;
    let showModal = false;

    async function upload_file(event) {
        try {
            console.log(selected_files);
            let task = handle_file_upload(topic, selected_files);
            not_uploading = false;
            await task;
            not_uploading = true;
        } catch (e) {
            //not_uploading = true;
            console.error(e);
            dispatch('app-error', { message: e.message });
        }
        //window.location.reload();
    }

    onMount(() => {
        imgs = get_image_names(topic).catch((e) => {
            console.error(e);
            dispatch('app-error', { message: e.message });
        });
    });
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
    <!-- on click open a modal to add tags -->
    <a href="javascript:void(0)" on:click={() => showModal = true}>Settings</a>
    <!-- Visible button that triggers the hidden input -->
    <a on:click={() => document.getElementById('file').click()}>Upload</a>
</Nav>
<ErrorMessage />

{#if showModal}
    <Modal on:close={() => showModal = false}>
        <TopicSettings topic={topic} />
    </Modal>
{/if}

{#if not_uploading}
<div class="grid">
    {#await imgs}
        <p>loading...</p>
    {:then imgs}
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
    {:catch error}
        <p>error: {error.message}</p>
    {/await}
</div>
{:else}
    <Uploading />
{/if}
