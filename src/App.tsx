import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import { Delete as DeleteIcon } from '@mui/icons-material';
import {
	Button,
	Container,
	IconButton,
	List,
	ListItem,
	ListItemText,
	TextField,
} from '@mui/material';

function App() {
	const [todos, setTodos] = useState<any[]>([]);
	const [newTodo, setNewTodo] = useState('');

	useEffect(() => {
		refreshTodos();
		listen('client:refresh', refreshTodos);
	}, []);

	return (
		<Container>
			<TextField value={newTodo} onChange={(e) => setNewTodo(e.target.value)} />
			<Button onClick={addTodo}>
				Add Todo
			</Button>

			<List>
				{todos.map((todo) => (
					<ListItem
						key={todo.id}
						secondaryAction={
							<IconButton
								edge="end"
								onClick={() => deleteTodo(todo.id)}
							>
								<DeleteIcon />
							</IconButton>
						}
					>
						<ListItemText>
							{todo.label}
						</ListItemText>
					</ListItem>
				))}
			</List>
		</Container>
	);

	async function addTodo() {
		// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

		await invoke('add_todo', { todo: { label: newTodo } });

		setNewTodo('');

		refreshTodos();
	}

	async function deleteTodo(id: string) {
		await invoke('delete_todo', { id });
		refreshTodos();
	}

	async function refreshTodos() {
		setTodos(await invoke<any[]>('get_todos'));
	}
}

export default App;
