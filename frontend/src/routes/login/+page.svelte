<script lang="ts">
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let email = '';
  let password = '';
  let error = '';
  let loading = false;

  async function handleLogin() {
    loading = true;
    error = '';
    try {
      const response = await api.post('/auth/login', { email, password });
      auth.set({
        user: response.user,
        token: response.tokens.access_token,
        isAuthenticated: true
      });
      goto('/');
    } catch (e: any) {
      error = e.message || 'Login failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center bg-muted/40 px-4">
  <div class="w-full max-w-sm space-y-6 rounded-lg border bg-card p-6 shadow-sm">
    <div class="space-y-2 text-center">
      <h1 class="text-3xl font-bold">HEDTRONIX</h1>
      <p class="text-muted-foreground">Healthcare Operating System</p>
    </div>
    
    {#if error}
      <div class="rounded-md bg-destructive/15 p-3 text-sm text-destructive">
        {error}
      </div>
    {/if}

    <form on:submit|preventDefault={handleLogin} class="space-y-4">
      <div class="space-y-2">
        <label for="email" class="text-sm font-medium">Email</label>
        <input 
          id="email"
          type="email" 
          bind:value={email}
          class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          placeholder="doctor@hedtronix.com"
          required
        />
      </div>
      <div class="space-y-2">
        <label for="password" class="text-sm font-medium">Password</label>
        <input 
          id="password"
          type="password" 
          bind:value={password}
          class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          required
        />
      </div>
      <button 
        type="submit" 
        disabled={loading}
        class="inline-flex h-10 w-full items-center justify-center whitespace-nowrap rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground ring-offset-background transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
      >
        {loading ? 'Signing in...' : 'Sign In'}
      </button>
    </form>
  </div>
</div>
