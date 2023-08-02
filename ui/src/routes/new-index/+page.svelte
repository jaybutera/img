<script>
    import { create_index } from "$lib/img.ts";
    let name;
    let topics;
    let submit_result;

    function parse_topics_list(topics) {
        return topics.split(",").map((topic) => topic.trim());
    }
    async function submit_index() {
        submit_result = await create_index(name, parse_topics_list(topics));
    }
</script>

<Nav></Nav>

{#if submit_result}
    <p>{submit_result}</p>
{/if}
<!-- input name and a list of topics -->
<input bind:value={name} type="text" name="name" placeholder="index name" />
<input bind:value={topics} type="text" name="topics" placeholder="topic-1, topic-2, ..." />

<!-- submit button, create_index takes the name and topics -->
<button on:click={submit_index}>Create Index</button>
