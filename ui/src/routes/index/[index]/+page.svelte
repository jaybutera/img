<script>
    import Nav from "../../components/Nav.svelte";
    import { img_server, handle_file_upload } from "$lib/img.ts";
    import { goto } from "$app/navigation";
    import Uploading from '../../components/Uploading.svelte';
    export let data;
    const index = data.index_name;
    const topics = data.topics;
</script>

<style>
</style>

<Nav>
</Nav>

<ul>
    {#each topics as topic}
        <li>
            <a href="/{index}/{topic}">{topic}</a>
        </li>
    {/each}
</ul>
