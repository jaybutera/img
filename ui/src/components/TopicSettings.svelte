<script>
    import { create_index, get_tags } from "$lib/img.ts";
    import Nav from './Nav.svelte';
    export let topic;
    let topics;
    let submit_result;

    function parse_topics_list(topics) {
        return topics.split(",").map((topic) => topic.trim());
    }
    async function submit_index() {
        submit_result = await create_index(topic, parse_topics_list(topics));
    }
</script>

<div class="vert-new-form">
    <div class="new-form">
        <h1>Add Tags</h1>
        <h2>This doesn't work yet...</h2>
        <!-- list of tags -->
        <h3>Current Tags</h3>
        <ul>
            {#await get_tags(topic)}
                <li>loading...</li>
            {:then tags}
                {#each tags as tag}
                    <li>{tag}</li>
                {/each}
            {:catch error}
                <li>error: {error.message}</li>
            {/await}
        </ul>

        <input bind:value={topics} type="text" class="nt-field nt-form" name="topics" placeholder="topic-1, topic-2, ..." />

        <button class="nt-form submit-form" on:click={submit_index}>Add Tag</button>
    </div>
</div>
