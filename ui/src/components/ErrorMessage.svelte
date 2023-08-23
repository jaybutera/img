<script>
    import { onMount } from 'svelte';

    let messages = [];

    onMount(() => {
        const handler = (e) => {
            messages.push(e);
        };

        document.addEventListener('app-error', handler);
        return () => {
            document.removeEventListener('app-error', handler);
        };
    });
</script>

{#each messages as message (message.id)}
    <div class="error">
        <h2>{message.text}</h2>
    </div>
{/each}
