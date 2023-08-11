<script>
    import Nav from "../../../components/Nav.svelte";
    import { img_server, handle_file_upload } from "$lib/img.ts";
    import { goto } from "$app/navigation";
    import Uploading from '../../../components/Uploading.svelte';
    export let data;
    const index = data.index_name;
    const topics = data.topics;
    const image_names = data.image_names;
</script>

<style>
    .media {
        max-width: 100%;
    }
    .center-container {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
    }
    .sub-container {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
    }
    .container {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        max-width: 66%;
    }
</style>

<Nav>
</Nav>

<div class="center-container">
<div class="container">
    <!-- zip topic and image_names together -->
    {#each topics as topic, i}
            <a href="/{topic}">
                <div class="sub-container">
                    <h2>{topic.replaceAll('-', ' ')}</h2>
                    <img class="media" src="{img_server}/img/{image_names[i]}"/>
                </div>
            </a>
    {/each}
</div>
</div>
