import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import {
	Button,
	Container,
	Typography,
} from '@mui/material';

interface Test {
	foo: string;
}

function App() {
	const [greetMsg, setGreetMsg] = useState<Test | null>(null);
	const [scanning, setScanning] = useState(false);

	async function greet() {
		setScanning(true);
		// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
		setGreetMsg(await invoke('greet', { name: 'Foo' }));
		setScanning(false);
	}

	return (
		<Container>
			<Button disabled={scanning} onClick={greet}>
				{scanning ? 'Scanning...' : 'Scan'}
			</Button>

			<Typography>{greetMsg?.foo}</Typography>
		</Container>
	);
}

export default App;
