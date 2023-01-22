import * as React from 'react';
import Box from '@mui/material/Box';
import FormControl from '@mui/material/FormControl';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import Stack from '@mui/material/Stack';
import PlayArrowRounded from '@mui/icons-material/PlayArrowRounded';
import IconButton from '@mui/material/IconButton';

import { invoke } from '@tauri-apps/api'

export const AppSettings = () => {
    // Ignore right click on setting view
    document.addEventListener(
        "contextmenu",
        (event) => {
            console.log(event);
            event.preventDefault();
            },
        { capture: true }
    );

    const [voiceState, setVoiceState] = React.useState("0");

    const onVoiceChange = (event: SelectChangeEvent) => {
        const value = event.target.value as string
        console.log(value);

        setVoiceState(value);
        //invoke("cmd_set_notification", {notification: index: value});
        //localStorage.setItem("sstimer-Voice", JSON.stringify(value));
    };

    const onPlayClick = () => {
        invoke("cmd_play_voice", {index: voiceState});
    };

    return (
        <Box margin={3} sx={{ justifyContent: 'center' }}>
            <Stack direction="row" spacing={2}>
                <FormControl size="small" >
                  <Select
                    labelId="voicelabel"
                    id="voice-select"
                    value={voiceState}
                    onChange={onVoiceChange}
                  >
                    <MenuItem value={0}>COEIROINK:おふとんP</MenuItem>
                    <MenuItem value={1}>COEIROINK:KANA</MenuItem>
                    <MenuItem value={2}>COEIROINK:MANA</MenuItem>
                    <MenuItem value={3}>COEIROINK:つくよみちゃん</MenuItem>
                    <MenuItem value={4}>VOICEVOX:四国めたん</MenuItem>
                    <MenuItem value={5}>VOICEVOX:ずんだもん</MenuItem>
                  </Select>
                </FormControl>
                <IconButton onClick={onPlayClick} sx={{ border: 1 }} aria-label="play">
                    <PlayArrowRounded />
                </IconButton>
            </Stack>
        </Box>
    );

}

export default AppSettings;
