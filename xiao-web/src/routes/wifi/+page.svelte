<script lang="ts">
	import { onMount } from 'svelte';

	let ssid = '';
	let psk = '';
	let security = 'WPA2';
	let showPassword = false;

	const securityOptions = [
		{ label: 'Open (No Security)', value: 'OPEN' },
		{ label: 'WEP', value: 'WEP' },
		{ label: 'WPA/WPA2-Personal', value: 'WPA2' },
		{ label: 'WPA3-Personal', value: 'WPA3' }
	];

	function submitForm() {
		alert(`SSID: ${ssid}\nPSK: ${psk}\nSecurity: ${security}`);
	}
</script>

<div class="min-h-screen flex flex-col items-center justify-center bg-base-200">
	<div class="w-full max-w-md p-8 rounded-box shadow-lg bg-base-100">
		<h1 class="text-2xl font-bold mb-6 text-center">WiFi Network Configuration</h1>
		<form on:submit|preventDefault={submitForm} class="flex flex-col gap-4">
			<div class="form-control">
				<label class="label">
					<span class="label-text">SSID</span>
				</label>
				<input type="text" class="input input-bordered" bind:value={ssid} required placeholder="Enter SSID" />
			</div>
			<div class="form-control">
				<label class="label">
					<span class="label-text">Security</span>
				</label>
				<select class="select select-bordered" bind:value={security}>
					{#each securityOptions as option}
						<option value={option.value}>{option.label}</option>
					{/each}
				</select>
			</div>
			{#if security !== 'OPEN'}
			<div class="form-control">
				<label class="label">
					<span class="label-text">Password (PSK)</span>
				</label>
				<div class="relative flex items-center">
					<input type={showPassword ? 'text' : 'password'} class="input input-bordered flex-1 pr-24 z-10" bind:value={psk} required placeholder="Enter password" minlength="8" />
					<button type="button" class="btn btn-sm btn-ghost w-20 absolute right-2 z-20" on:click={() => showPassword = !showPassword} tabindex="-1">
						{showPassword ? 'Hide' : 'Show'}
					</button>
				</div>
			</div>
			{/if}
			<button class="btn btn-primary mt-4" type="submit">Save Configuration</button>
		</form>
	</div>
</div>
