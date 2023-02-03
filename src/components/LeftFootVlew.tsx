import React from 'react';

import { emit } from '@tauri-apps/api/event'

import Divider from '@mui/material/Divider';
import Box from '@mui/material/Box';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import LogoutIcon from '@mui/icons-material/Logout';
import LoginIcon from '@mui/icons-material/Login';


export const LeftFoot = () => {
  const [loggedin, setLoggedin] = React.useState(true);
  const onLogoutClick = () => {
      setLoggedin(!loggedin);
      if (loggedin) {
        emit('tauri://frontend/token-unregister');
      }
  };

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

