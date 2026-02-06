<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Editor } from '@tiptap/core';
    import StarterKit from '@tiptap/starter-kit';
    import Placeholder from '@tiptap/extension-placeholder';
    import { Button } from "$lib/components/ui/button";
    import { Bold, Italic, List, ListOrdered, Quote, Heading1, Heading2, RotateCcw, RotateCw } from 'lucide-svelte';

    export let content = '';
    export let editable = true;
    export let placeholder = 'Write something...';

    let element: HTMLElement;
    let editor: Editor;

    onMount(() => {
        editor = new Editor({
            element: element,
            extensions: [
                StarterKit,
                Placeholder.configure({
                    placeholder,
                }),
            ],
            content,
            editable,
            onTransaction: () => {
                // Force reactivity
                editor = editor;
            },
            onUpdate: ({ editor }) => {
                content = editor.getHTML();
            },
            editorProps: {
                attributes: {
                    class: 'prose prose-sm sm:prose lg:prose-lg xl:prose-2xl mx-auto focus:outline-none min-h-[150px] p-4'
                }
            }
        });
    });

    onDestroy(() => {
        if (editor) {
            editor.destroy();
        }
    });

    $: if (editor && content !== editor.getHTML()) {
        // Handle external content changes (careful with loops)
        // editor.commands.setContent(content);
    }
</script>

<div class="border rounded-md bg-background w-full">
    {#if editor && editable}
        <div class="flex flex-wrap items-center gap-1 p-2 border-b bg-muted/40">
            <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().toggleBold().run()} 
                class={editor.isActive('bold') ? 'bg-muted' : ''}>
                <Bold class="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().toggleItalic().run()} 
                class={editor.isActive('italic') ? 'bg-muted' : ''}>
                <Italic class="h-4 w-4" />
            </Button>
            <div class="w-px h-6 bg-border mx-1"></div>
            <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().toggleHeading({ level: 1 }).run()} 
                class={editor.isActive('heading', { level: 1 }) ? 'bg-muted' : ''}>
                <Heading1 class="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().toggleHeading({ level: 2 }).run()} 
                class={editor.isActive('heading', { level: 2 }) ? 'bg-muted' : ''}>
                <Heading2 class="h-4 w-4" />
            </Button>
            <div class="w-px h-6 bg-border mx-1"></div>
            <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().toggleBulletList().run()} 
                class={editor.isActive('bulletList') ? 'bg-muted' : ''}>
                <List class="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().toggleOrderedList().run()} 
                class={editor.isActive('orderedList') ? 'bg-muted' : ''}>
                <ListOrdered class="h-4 w-4" />
            </Button>
            <div class="w-px h-6 bg-border mx-1"></div>
            <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().toggleBlockquote().run()} 
                class={editor.isActive('blockquote') ? 'bg-muted' : ''}>
                <Quote class="h-4 w-4" />
            </Button>
            <div class="ml-auto flex gap-1">
                <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().undo().run()} disabled={!editor.can().undo()}>
                    <RotateCcw class="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" on:click={() => editor.chain().focus().redo().run()} disabled={!editor.can().redo()}>
                    <RotateCw class="h-4 w-4" />
                </Button>
            </div>
        </div>
    {/if}
    
    <div bind:this={element} class="min-h-[200px] w-full" />
</div>

<style>
    /* Add basic prose styles if @tailwindcss/typography is not installed */
    :global(.ProseMirror) {
        outline: none;
    }
    :global(.ProseMirror p.is-editor-empty:first-child::before) {
        color: #adb5bd;
        content: attr(data-placeholder);
        float: left;
        height: 0;
        pointer-events: none;
    }
</style>
