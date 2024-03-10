import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import {
	Button,
	Container,
	Typography,
} from '@mui/material';

function App() {
	const [todos, setTodos] = useState<any[]>([]);

	useEffect(() => {
		(async () => {
			setTodos(await invoke('get_todos'));
		})();
	}, []);

	async function addTodo() {
		// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

		console.log(111);

		await invoke('add_todo', { todo: { label: 'From Tauri!' } });
		console.log(222);

		const x = await invoke<any[]>('get_todos');

		console.log('x', x);

		setTodos(x);
	}

	return (
		<Container>
			<Button onClick={addTodo}>
				Add Todo
			</Button>

			<Typography>{JSON.stringify(todos)}</Typography>
		</Container>
	);
}

export default App;
