import { ThemeProvider } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import { appWindow } from '@tauri-apps/api/window';
import { useMediaQuery } from '@mui/material';
import React, { useEffect } from 'react';
import LanguageSelector from './components/LanguageSelector';
import TargetArea from './components/TargetArea';
import SourceArea from './components/SourceArea';
import TopBar from './components/TopBar';
import { light, dark } from '../themes';
import { get } from '../main';

export default function Translator() {
    const theme = get('theme') ?? 'auto';
    const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');
    useEffect(() => {
        if (appWindow.label != 'util') {
            appWindow.show();
            appWindow.setFocus();
        }
    }, []);
    return (
        <ThemeProvider theme={theme == 'auto' ? (prefersDarkMode ? dark : light) : theme == 'dark' ? dark : light}>
            <CssBaseline />
            <div
                data-tauri-drag-region
                className='titlebar'
            />
            <TopBar />
            <SourceArea />
            <LanguageSelector />
            <TargetArea />
        </ThemeProvider>
    );
}
