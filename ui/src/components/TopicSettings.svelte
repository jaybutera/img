<script>
    import { create_index, get_tags, add_tag, rm_tag } from "$lib/img.ts";
    import Nav from './Nav.svelte';
    export let topic;
    let tag;
    let submit_result;
    let tags = get_tags(topic);

    async function rm_tag_handler(tag) {
        let res = await rm_tag(topic, tag);
        if (!res.ok) {
            submit_result = res;
        }
        // Reload
        tags = await get_tags(topic);
    }
    async function add_tag_handler() {
        let res = await add_tag(topic, tag);
        if (!res.ok) {
            submit_result = res;
        }
        // Reload
        tags = await get_tags(topic);
    }
</script>

<style>
    .tag-list {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
    }
    .tag {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
    }
    .tag span {
        margin-right: 0.5em;
        padding: 0.5em;
        border: 1px solid black;
        border-radius: 0.3em;
    }
</style>

<div class="vert-new-form">
    <div class="new-form">
        <h1>Add Tags</h1>
        <h2>This doesn't work yet...</h2>
        <!-- list of tags -->
        <h3>Current Tags</h3>
        <ul>
            {#await tags}
                <li>loading...</li>
            {:then tags}
                <div class="tag-list">
                    {#each tags as tag}
                        <div class="tag">
                            <span>
                                {tag}
                                <button on:click={() => rm_tag_handler(tag)}>X</button>
                            </span>
                        </div>
                    {/each}
                </div>
            {:catch error}
                <li>error: {error.message}</li>
            {/await}
        </ul>

        <div class="add-form">
            <input bind:value={tag} type="text" class="nt-field nt-form" name="topics" placeholder="topic-1, topic-2, ..." />
            <button class="nt-form submit-form" on:click={() => add_tag_handler()}>Add Tag</button>
            {#if submit_result}
                <p>{submit_result}</p>
            {/if}
        </div>
    </div>
</div>
