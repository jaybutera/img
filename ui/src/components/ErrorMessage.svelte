<script>
    import { onMount } from 'svelte';

    let messages = [];

    onMount(() => {
        const handler = (e) => {
            messages.push(e.detail);
        };

        document.addEventListener('app-error', handler);
        return () => {
            document.removeEventListener('app-error', handler);
        };
    });
</script>

{#each messages as message (message.id)}
    <div class="error">
        {message.text}
    </div>
{/each}
