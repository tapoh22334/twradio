import React from 'react';

import { Link } from 'react-router-dom';

import { invoke } from '@tauri-apps/api'

import List from '@mui/material/List';
import Divider from '@mui/material/Divider';
import Box from '@mui/material/Box';
import InfoIcon from '@mui/icons-material/Info';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import SettingsIcon from '@mui/icons-material/Settings';
import AbcIcon from '@mui/icons-material/Abc';


export const Drawer = () => {
  const onTweetsClick = () => {
    //setPaused(false);
    //invoke('set_paused', {paused: false});
  }

  const onSearchClick = () => {
    //setPaused(false);
    //invoke('set_paused', {paused: false});
  }

  const onSettingsClick = () => {
    //setPaused(true);
    //invoke('set_paused', {paused: true});
  }

  const onLicensesClick = () => {
    //setPaused(true);
    //invoke('set_paused', {paused: true});
  }

  return (
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
        <ListItemButton onClick={onTweetsClick}>
          <ListItemIcon>
            <AbcIcon />
          </ListItemIcon>
          <ListItemText primary='Timeline' />
        </ListItemButton>
      </ListItem>
      </Link>

      <Divider />

      <Link style={{ textDecoration: 'none' }} to="search">
      <ListItem
        key='Search'
        disablePadding
        >
        <ListItemButton onClick={onSearchClick}>
          <ListItemIcon>
            <AbcIcon />
          </ListItemIcon>
          <ListItemText primary='Search' />
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
};
