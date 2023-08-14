<script>
    import { create_index } from "$lib/img.ts";
    import Nav from '../../components/Nav.svelte';
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
<div class="vert-new-form">
    <div class="new-form">
        <input bind:value={name} type="text" class="nt-field nt-form" name="name" placeholder="index name" />
        <input bind:value={topics} type="text" class="nt-field nt-form" name="topics" placeholder="topic-1, topic-2, ..." />

        <!-- submit button, create_index takes the name and topics -->
        <button class="nt-form submit-form" on:click={submit_index}>Create Index</button>
    </div>
</div>
