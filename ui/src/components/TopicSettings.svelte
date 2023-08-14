<script>
    import { create_index } from "$lib/img.ts";
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
        <input bind:value={topics} type="text" class="nt-field nt-form" name="topics" placeholder="topic-1, topic-2, ..." />

        <button class="nt-form submit-form" on:click={submit_index}>Create Index</button>
    </div>
</div>
