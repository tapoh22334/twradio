import React from 'react';

import { AppContext } from '../AppContext'

import { invoke } from '@tauri-apps/api'

import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Slider from '@mui/material/Slider';
import VolumeUp from '@mui/icons-material/VolumeUp';
import IconButton from '@mui/material/IconButton';
import PauseRounded from '@mui/icons-material/PauseRounded';
import PlayArrowRounded from '@mui/icons-material/PlayArrowRounded';
import FastForwardRounded from '@mui/icons-material/FastForwardRounded';
import CenterFocusStrongIcon from '@mui/icons-material/CenterFocusStrong';
import CenterFocusWeakIcon from '@mui/icons-material/CenterFocusWeak';


export const TWAppBar = () => {
  const {focusTweetIdPair, tweetListPair, pausedPair, focusedPair} = React.useContext(AppContext)
  const [focusTweetId, setFocusTweetId] = focusTweetIdPair;
  const [tweetList, setTweetList] = tweetListPair;
  const [paused, setPaused] = pausedPair;
  const [focused, setFocused] = focusedPair;

  const onPauseResumeClick = () => {
    setPaused(!paused);
    invoke('set_paused', {paused: !paused});
  }

  const onSkipClick = () => {
    const index = tweetList.findIndex((elem) => elem.tweet_id === focusTweetId);
    let id;
    if (index in tweetList) {
        id = tweetList[index + 1]?.tweet_id;
    } else {
        id = "";
    }

    invoke('jump', {twid: id});
  }

  const onFocusClick = () => {
     setFocused(!focused);
  }

  const [volume, setVolume] = React.useState(() => {
    const json = localStorage.getItem("volume");
    const parsedInitVolume = json === null ? null : JSON.parse(json);
    const initVolume = parsedInitVolume === null ? 80 : parsedInitVolume;

    invoke('set_volume', {volume: initVolume as number});
    return initVolume;
  });

  React.useEffect(() => {
    console.log(volume);
    invoke('set_volume', {volume: volume as number});
    localStorage.setItem("volume", JSON.stringify(volume as number));
  }, [volume]);

  const onVolumeChange = (_: Event, newValue: number | number[]) => {
    console.log(newValue);
    setVolume(newValue as number);
  };

  return (
  <AppBar className="Head" position="fixed"
      sx={{
        width: `calc(100% - var(--drawer-width))`,
        ml: `var(--drawer-width)`,
      }}>

      <Toolbar>
          <IconButton
              color="inherit"
              onClick={onPauseResumeClick}
          >
              {paused ? <PlayArrowRounded /> : <PauseRounded />}
          </IconButton>

          <IconButton
              color="inherit"
              onClick={onSkipClick}>
              <FastForwardRounded />
          </IconButton>

          <IconButton
              color="inherit"
              onClick={onFocusClick}>
              { focused? <CenterFocusStrongIcon/> : <CenterFocusWeakIcon/> }
          </IconButton>

          <VolumeUp 
            sx={{ mr: 1 }}
          />
          <Slider value={volume}
              onChange={onVolumeChange}
              min={0}
              max={100}
              sx={{ width: '40%', color: "inherit"}}/>

      </Toolbar>
  </AppBar>
 );
}

