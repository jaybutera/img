<script>
    import { onMount } from 'svelte';
    import { get_pubkey } from '$lib/img.ts';

    let public_key;

    onMount(() => async () => {
        async function init() {
            public_key = await get_pubkey();
        }
        init().catch(e => console.error(e));
    });
</script>

<style>
.flex {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
}
.flex-left {
    display: flex;
    justify-content: space-between;
    align-items: center;
}
.flex-right {
    display: flex;
    justify-content: space-between;
    align-items: right;
    padding-right: 1rem;
}
</style>

<div class="topnav">
    <div class="flex">
        <div class="flex-left">
            <a href="https://github.com/jaybutera/img">About</a>
            <a href="/new-topic">New Collection</a>
        <slot />
        </div>
        <div class="flex-right">
            {#await get_pubkey() then public_key}
                {#if public_key}
                    <a href="/import-key">Public Key: {public_key.slice(0,8)}...</a>
                {:else}
                    <a href="/import-key">Login</a>
                {/if}
            {/await}
        </div>

    <!--<a href="/new-index">New Index</a>-->
    </div>
</div>
