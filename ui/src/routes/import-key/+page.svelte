<script>
    import Nav from "../../components/Nav.svelte";
    import { Buffer } from 'buffer';
    import { get_challenge, authenticate } from '$lib/img.ts';
    import { createEventDispatcher } from 'svelte';
    const dispatch = createEventDispatcher();

    let secret_key = "";
    let authenticated = false;

    // save secret key to browser store
    async function save_secret() {
        if (secret_key.length != 44) {
            console.error('Private key is not 32 bytes long.');
            dispatch('error', { message: 'Private key is not 32 bytes long.' });
            return;
        }
        if (!secret_key.match(/^[a-zA-Z0-9+/]+={0,2}$/)) {
            console.error('Private key is not base64 encoded.');
            dispatch('error', { message: 'Private key is not base64 encoded.' });
            return;
        }

        localStorage.setItem("private_key", secret_key);
        await auth();
    }

    async function auth() {
        // zeroed out uint8array for testing (32 bytes)
        //let challenge = new Uint8Array(32);
        // dispatch any thrown errors
        try {
            let challenge = await get_challenge();
            console.log('challenge', challenge);
            await authenticate(challenge, secret_key);
        } catch (e) {
            console.error(e);
            dispatch('error', { message: e.message });
        }
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
    <button class="nt-form" on:click={save_secret}>Import</button>
{/if}
