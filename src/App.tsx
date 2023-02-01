import React from 'react';
import './App.css';

import { BrowserRouter, Routes, Route, Link } from 'react-router-dom';

import { invoke } from '@tauri-apps/api'
import { listen, emit } from '@tauri-apps/api/event'
import { exit } from '@tauri-apps/api/process';
import { TweetLi, TweetProps, TweetLiProps } from './components/TweetCard'
import Licenses from './components/LicenseView'

import List from '@mui/material/List';
import Divider from '@mui/material/Divider';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Slider from '@mui/material/Slider';
import VolumeUp from '@mui/icons-material/VolumeUp';
import IconButton from '@mui/material/IconButton';
import PauseRounded from '@mui/icons-material/PauseRounded';
import PlayArrowRounded from '@mui/icons-material/PlayArrowRounded';
import FastForwardRounded from '@mui/icons-material/FastForwardRounded';
import AdjustIcon from '@mui/icons-material/Adjust';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import InfoIcon from '@mui/icons-material/Info';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import SettingsIcon from '@mui/icons-material/Settings';
import LogoutIcon from '@mui/icons-material/Logout';
import LoginIcon from '@mui/icons-material/Login';
import AbcIcon from '@mui/icons-material/Abc';
import FormControl from '@mui/material/FormControl';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import Stack from '@mui/material/Stack';


type ViewElements = {
    tweet_id: string,
    created_at: string,
    text: string,
    name: string,
    username: string,
    profile_image_url: string,
}

function App() {
  const [focusTwid, setFocusTwid] = React.useState<string>(()=>{
    return ""
  });
  React.useEffect(() => {
    scrollToFocus()
  }, [focusTwid]);

  const [tweetList, setTweetList] = React.useState<Array<TweetProps>>(()=>{
    return []
  });

  const onVolumeChange = (event: Event, newValue: number | number[]) => {
    setVolume(newValue as number);
    invoke('set_volume', {volume: newValue as number});
    localStorage.setItem("volume", JSON.stringify(newValue as number));
  };

  const [paused, setPaused] = React.useState(false);
  const onPauseResumeClick = () => {
    setPaused(!paused);
    invoke('set_paused', {paused: !paused});
  }

  const [inroot, setInroot] = React.useState(true);
  const onRootClick = () => {
    setInroot(true);
    setPaused(false);
    invoke('set_paused', {paused: false});
  }
  React.useEffect(() => {
    scrollToFocus()
  }, [inroot]);

  const onSettingsClick = () => {
    setPaused(true);
    setInroot(false);
    invoke('set_paused', {paused: true});
  }

  const onLicensesClick = () => {
    setPaused(true);
    setInroot(false);
    invoke('set_paused', {paused: true});
  }

  const scrollToFocus = () => {
    const targetEl = document.getElementById(focusTwid)
    if (targetEl && window.location.pathname === "/") {
        targetEl?.scrollIntoView({ behavior: 'smooth' })
        console.log(focusTwid);
    }
  }
  const onFocusClick = () => {
    scrollToFocus()
  }

  const [loggedin, setLoggedin] = React.useState(true);
  const onLogoutClick = () => {
      setLoggedin(!loggedin);
      if (loggedin) {
        emit('tauri://frontend/token-unregister');
      }
  };

  const onSkipClick = () => {
    const index = tweetList.findIndex((elem) => elem.tweet_id === focusTwid);
    let id;
    if (index in tweetList) {
        id = tweetList[index + 1]?.tweet_id;
    } else {
        id = "";
    }

    invoke('jump', {twid: id});
  }

  const [volume, setVolume] = React.useState(() => {
    const json = localStorage.getItem("volume");
    const parsedInitVolume = json === null ? null : JSON.parse(json);
    const initVolume = parsedInitVolume === null ? 80 : parsedInitVolume;

    invoke('set_volume', {volume: initVolume as number});
    return initVolume;
  });

  // Used in setting context
    type SpeakerInfo = {
        addr: string,
        engine: string,
        name: string,
        style: string,
        speaker: string,
    }

    const [AuthErr, setAuthErr] = React.useState<string>(()=>{ return ""; });

    listen<string>('tauri://frontend/authorization-failed', (event)=> {
        const errmsg: string = event.payload;
        setAuthErr(errmsg);

        console.log(errmsg);
    });

    const [NoTTSErr, setNoTTSErr] = React.useState<string>(()=>{ return ""; });

    listen<string>('tauri://frontend/no-voicegen-found', (event)=> {
        const errmsg: string = event.payload;
        setNoTTSErr(errmsg);

        console.log(errmsg);
    });

    const [TTSErr, setTTSErr] = React.useState<string>(()=>{ return ""; });

    listen<string>('tauri://frontend/tts-failed', (event)=> {
        const errmsg: string = event.payload;
        setTTSErr(errmsg);

        console.log(errmsg);
    });

    const [otherErr, setOtherErr] = React.useState<string>(()=>{ return ""; });

    listen<string>('tauri://frontend/other-error', (event)=> {
        const errmsg: string = event.payload;
        setOtherErr(errmsg);

        console.log(errmsg);
    });

    const to_unique_string = (speaker: SpeakerInfo) => {
        return speaker.addr + "/" + speaker.speaker;
    };

    const [speaker, setSpeaker] = React.useState(() => {
        const json = localStorage.getItem("speaker");
        const parsedInitSpeaker = json === null ? null : JSON.parse(json);
        const initSpeaker = parsedInitSpeaker === null ? "127.0.0.1:50021/0" : parsedInitSpeaker;

        return initSpeaker;
    });

    const [speakerList, setSpeakerList] = React.useState<Array<SpeakerInfo>>(()=>{
      return []
    });

    const onSpeakerChange = (event: SelectChangeEvent) => {
        const value = event.target.value as string
        console.log(value);

        setSpeaker(value);
        const index = speakerList.findIndex((e) => to_unique_string(e) === value);
        console.log(speakerList[index]);
        invoke("set_speaker", {speaker: speakerList[index]});
        localStorage.setItem("speaker", JSON.stringify(value));
    };

    listen<Array<SpeakerInfo>>('tauri://frontend/speakers-register', (event)=> {
        const speakers: Array<SpeakerInfo> = event.payload;
        console.log(speakers);

        speakerList.splice(0);
        for (let sp of speakers) {
            speakerList.push(
                {
                    addr: sp.addr,
                    engine: sp.engine,
                    name: sp.name,
                    style: sp.style,
                    speaker: sp.speaker,
                }
            )
        }

        const index = speakerList.findIndex((e) => to_unique_string(e) === speaker);
        invoke("set_speaker", {speaker: speakerList[index]});

        setSpeakerList([...speakerList]);
    });


    const AppSettings = () => {
        return (
            <Box margin={2}>
                <Typography gutterBottom>
                  声
                </Typography>
                <FormControl size="small" >
                  <Select
                    labelId="voicelabel"
                    id="voice-select"
                    value={speaker}
                    onChange={onSpeakerChange}
                  >
                    {
                        speakerList.length > 0 &&
                        speakerList.map((speaker, index) => {
                            return (<MenuItem value={to_unique_string(speaker)}>{speaker.engine}:{speaker.name}[{speaker.style}]</MenuItem>)
                        })
                    }
                  </Select>
                </FormControl>
            </Box>
        );

    }
  // <- Used in setting context

  React.useEffect(() => {
    listen('tauri://frontend/token-register', (event)=> {
        console.log(event);
        localStorage.setItem("token", JSON.stringify(event.payload));
    });

    listen('tauri://frontend/token-unregister', (event)=> {
        console.log(event);
        localStorage.removeItem("token");
        exit(1);
    });

    listen('tauri://frontend/token-request', (event)=> {
        const token = localStorage.getItem("token")
        if (token) {
            const json = JSON.parse(token);
            emit('tauri://backend/token-response', json);

            console.log(json);
        } else {
            emit('tauri://backend/token-response');

            console.log("return none");
        }
    });

    listen<ViewElements>('tauri://frontend/display/add', (event) => {
        const data: ViewElements = event.payload;
        tweetList.push(
            {tweet_id: data.tweet_id,
            username: data.name,
            user_id: data.username,
            time: data.created_at,
            tweet: data.text,
            profile_image_url: data.profile_image_url
            }
        )
        setTweetList([...tweetList]);
        setAuthErr("");
    });

    listen<string>('tauri://frontend/display/delete', (event) => {
        const twid: string = event.payload;
        const index = tweetList.findIndex((elem) => elem.tweet_id === twid);
        tweetList.splice(index, 1);
        setTweetList([...tweetList]);
    });

    listen<string>('tauri://frontend/display/scroll', (event) => {
        const twid: string = event.payload;
        //const targetEl = document.getElementById(twid)
        //targetEl?.scrollIntoView({ behavior: 'smooth' })
        setFocusTwid(twid);
        console.log(twid);
    });

    console.log("invoke setup_app function");

    invoke('setup_app').then(() => console.log('setup_app complete'));
    // 'emit, listen' works correct from here !!
    emit('tauri://backend/ipc-init');

    listen('tauri://frontend/speakers-ready', (event)=> {
        console.log('tauri://frontend/speakers-ready');
        emit("tauri://backend/speakers-ready");
    });


  }, []) ;


    const drawerElements = () => (
    <Box
      sx={{ width: `var(--drawer-width)` }}
      role="presentation"
    >
      <List>

      <Divider />

      <Link style={{ textDecoration: 'none' }} to="/">
      <ListItem
        key='Timeline'
        disablePadding
        >
        <ListItemButton onClick={onRootClick}>
          <ListItemIcon>
            <AbcIcon />
          </ListItemIcon>
          <ListItemText primary='Timeline' />
        </ListItemButton>
      </ListItem>
      </Link>

      <Divider />

      <Link style={{ textDecoration: 'none' }} to="settings">
      <ListItem
        key='Settings'
        disablePadding
        >
        <ListItemButton onClick={onSettingsClick}>
          <ListItemIcon>
            <SettingsIcon />
          </ListItemIcon>
          <ListItemText primary='Settings' />
        </ListItemButton>
      </ListItem>
      </Link>

      <Divider />

      <Link style={{ textDecoration: 'none' }} to="licenses">
      <ListItem
        key='Information'
        disablePadding
        >
        <ListItemButton onClick={onLicensesClick}>
          <ListItemIcon>
            <InfoIcon />
          </ListItemIcon>
          <ListItemText primary='Information' />
        </ListItemButton>
      </ListItem>
      </Link>

      <Divider />


      </List>
    </Box>
    );

    const TWAppBar = () => {
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
                    disabled={!inroot}
                >
                    {paused ? (
                        <PlayArrowRounded />
                        ) : (
                        <PauseRounded />
                    )}
                </IconButton>

                <IconButton
                    color="inherit"
                    disabled={!inroot}
                    onClick={onSkipClick}>
                    <FastForwardRounded />
                </IconButton>

                <IconButton
                    color="inherit"
                    disabled={!inroot}
                    onClick={onFocusClick}>
                    <AdjustIcon />
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

    const Body = () => {
    return (
        <List
          sx={{
            //maxWidth: 360,
            bgcolor: 'background.paper',
          }}
        >
            {
                tweetList.length > 0 &&
                    tweetList.map((row) => {
                        return (
                         <React.Fragment>
                            <TweetLi
                                tweet_id={row.tweet_id}
                                username={row.username}
                                user_id={row.user_id}
                                time={row.time}
                                tweet={row.tweet}
                                profile_image_url={row.profile_image_url}
                                //focus={row.tweet_id === focusTwid ? true : false}
                                focus={false}
                                />
                            <Divider component="li" />
                         </React.Fragment>
                        )
                    })
            }
        </List>
     );
    }

    const LeftFoot = () => {
        return (
          <Box>
              <Divider />

                {loggedin ? 
                (
                    <ListItemButton onClick={onLogoutClick}>
                        <ListItemIcon>
                          <LogoutIcon />
                        </ListItemIcon>
                        <ListItemText primary='Logout' />
                    </ListItemButton>
                ) :
                (
                    <ListItemButton onClick={onLogoutClick}>
                        <ListItemIcon>
                          <LoginIcon />
                        </ListItemIcon>
                        <ListItemText primary='Login' />
                    </ListItemButton>
                )
                }
          </Box>
        );
    }

  return (
    <Box className="App" >
        <BrowserRouter>
        <Toolbar/>

        <Box sx={{ display: 'flex' }}>
            <TWAppBar/>

            <Box className="SideBar" >
                {drawerElements()}
            </Box>

            <Box className="Body" >
                <Routes>
                    <Route path={`/`} element={<Body />} />
                    <Route path={`settings`} element={<AppSettings />} />
                    <Route path={`licenses`} element={<Licenses />} />
                </Routes>
            </Box>
        </Box>

        <Box sx={{ display: 'flex' }}>
            <Box className="LeftFoot ">
                <LeftFoot/>
            </Box>
            <Box className="RightFoot" >
                {
                    AuthErr !== "" ? <Alert severity="warning">{AuthErr}</Alert> :<></>
                }
                {
                    otherErr !== "" ? <Alert severity="warning">{otherErr}</Alert> :<></>
                }
                {
                    NoTTSErr !== "" ? <Alert severity="warning">{NoTTSErr}</Alert> :<></>
                }
                {
                    TTSErr !== "" ? <Alert severity="warning">{TTSErr}</Alert> :<></>
                }
                <Alert severity="info">バグ報告等 Twitter @tapoh22334</Alert>
            </Box>
        </Box>

        </BrowserRouter>
    </Box>
  );
}

export default App;
