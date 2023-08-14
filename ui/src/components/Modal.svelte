<script>
    import { createEventDispatcher } from 'svelte';
    const dispatch = createEventDispatcher();

    function close() {
        dispatch('close');
    }
</script>

<style>
    .modal-background {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 9999;
    }

    .modal-content {
        background-color: white;
        padding: 2em;
        border-radius: 8px;
    }
</style>

<div class="modal-background" on:click={close}>
    <div class="modal-content" on:click={e => e.stopPropagation()}>
        <slot></slot>
        <button class="nt-form" on:click={close}>Close</button>
    </div>
</div>
