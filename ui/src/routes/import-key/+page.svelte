<script>
    import Nav from "../../components/Nav.svelte";
    import { Buffer } from 'buffer';
    import { save_privkey, get_challenge, authenticate, generate_key } from '$lib/img.ts';
    import { createEventDispatcher } from 'svelte';
    const dispatch = createEventDispatcher();

    let secret_key;
    let authenticated = false;
    let generated_sk = "";

    // save secret key to browser store
    async function save_secret() {
        save_privkey(secret_key);
        await auth();
        /*
        if (authenticated) {
            dispatch('authenticated', { message: 'Authenticated!' });
        }
        */
    }

    async function auth() {
        // zeroed out uint8array for testing (32 bytes)
        try {
            let challenge = await get_challenge();
            console.log('challenge', challenge);
            await authenticate(challenge);
            authenticated = true;
        } catch (e) {
            console.error(e);
            dispatch('error', { message: e.message });
        }
    }

    async function new_key() {
        generated_sk = await generate_key();
    }

</script>

<style>
</style>

<Nav />
{#if authenticated}
    <h1>You Are Authenticated!</h1>
    <h2>Sorry for the interruption and proceed :)</h2>
{:else}
    <input class="nt-form" bind:value={secret_key} type="text" id="secret" />
    <button class="nt-form" on:click={() => save_secret()}>Import</button>
{/if}

<div class="nt-form">
    <button class="nt-form" on:click={() => new_key()}>Generate New Secret Key</button>
    <p>Secret Key: {generated_sk}</p>
</div>
