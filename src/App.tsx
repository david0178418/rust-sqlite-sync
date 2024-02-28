import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import {
	Button, Container, Typography,
} from '@mui/material';

function App() {
	const [greetMsg, setGreetMsg] = useState('');

	async function greet() {
		// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
		setGreetMsg(await invoke('greet', { name: 'Foo' }));
	}

	return (
		<Container>
			<Button onClick={greet}>
				Greet
			</Button>

			<Typography>{greetMsg}</Typography>
		</Container>
	);
}

export default App;
