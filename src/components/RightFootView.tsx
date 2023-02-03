import React from 'react';

import { listen } from '@tauri-apps/api/event'

import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';


export const RightFoot = () => {
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

    return (
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
    );
}
