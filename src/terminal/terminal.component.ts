import {CommonModule} from '@angular/common';
import {FormsModule} from '@angular/forms';
import {Component, OnInit} from '@angular/core';
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";

@Component({
    selector: 'app-terminal',
    standalone: true,
    imports: [CommonModule, FormsModule],
    templateUrl: './terminal.component.html',
    styleUrl: './terminal.component.css'
})
export class TerminalComponent implements OnInit {
    path: string = '.';
    files: string[] = [];
    terminalOutput = '';
    inputText = '';

    ngOnInit() {
        this.setupTerminal();
    }

    async setupTerminal() {
        console.log('Setting up terminal');
        await invoke('async_create_shell');
        this.loadDirectory();

        listen<string>('terminal-output', (event) => {
            this.terminalOutput += event.payload;
        });

        listen<string>('shell-exit', () => {
            this.terminalOutput += '\n[Process exited]';
        });

    }

    async onEnter() {
        const input = this.inputText + '\n';
        this.inputText = '';
        console.log('Sending to shell:', input);
        await invoke('async_write_to_pty', {data: input});
    }

    loadDirectory() {
        invoke<string[]>('list_directory', {path: this.path})
            .then((result) => (this.files = result))
            .catch((err) => console.error('Directory fetch error:', err));
    }
}

