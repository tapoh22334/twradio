import React from 'react';

import { AppContext } from '../AppContext'
import { invoke } from '@tauri-apps/api'
import { listen} from '@tauri-apps/api/event'

import Slider from '@mui/material/Slider';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import FormControl from '@mui/material/FormControl';
import MenuItem from '@mui/material/MenuItem';

import Select, { SelectChangeEvent } from '@mui/material/Select';

export type SpeakerInfo = {
        addr: string,
        engine: string,
        name: string,
        style: string,
        speaker: string,
    }

export const toUniqueSpeakerId = (speaker: SpeakerInfo) => {
    return speaker.addr + "/" + speaker.speaker;
};

export const Settings = () => {
    const {speakerPair, speakerListPair, speechRatePair} = React.useContext(AppContext);
    const [speaker, setSpeaker] = speakerPair;
    const [speakerList, setSpeakerList] = speakerListPair;
    const [speechRate, setSpeechRate] = speechRatePair;

    const onSpeakerChange = (event: SelectChangeEvent) => {
        const value = event.target.value as string
        setSpeaker(value);
    };

    React.useEffect(() => {
        const index = speakerList.findIndex((e) => toUniqueSpeakerId(e) === speaker);
        invoke("set_speaker", {speaker: speakerList[index]});
        localStorage.setItem("speaker", JSON.stringify(speaker));

        console.log("speaker changed" + speaker);
    }, [speaker]);


    const onSpeechRateChange = (_: Event, value: number | number[]) => {
        console.log(value);
        setSpeechRate(value as number)
    }

    return (
        <Box>
            <Box margin={2}>
                <Typography gutterBottom>
                  声
                </Typography>
                <FormControl size="small" >
                  <Select
                    value={speaker}
                    onChange={onSpeakerChange}
                  >
                    {
                        speakerList.length > 0 &&
                        speakerList.map((speaker, _) => {
                            return (
                                <MenuItem value={toUniqueSpeakerId(speaker)}>
                                    {speaker.engine}:{speaker.name}[{speaker.style}]
                                </MenuItem>)
                        })
                    }
                  </Select>
                </FormControl>
            </Box>

            <Box margin={2}>
                <Typography gutterBottom>
                  話速
                </Typography>
                <Slider
                  step={0.01}
                  min={0.5}
                  max={2.00}
                  valueLabelDisplay="auto"
                  value={speechRate}
                  onChange={onSpeechRateChange}
                />
            </Box>
        </Box>
    );

}
