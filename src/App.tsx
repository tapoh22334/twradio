import React from "react";
import "./App.css";

import { BrowserRouter, Routes, Route, Link } from "react-router-dom";

import { invoke } from "@tauri-apps/api";
import { listen, emit } from "@tauri-apps/api/event";
import { exit } from "@tauri-apps/api/process";

import { AppContext } from "./AppContext";
import { Licenses } from "./components/LicenseView";
import { Drawer } from "./components/DrawerView";
import { TweetView } from "./components/TweetView";
import { Settings } from "./components/SettingsView";
import { TWAppBar } from "./components/TWAppBar";
import { LeftFoot } from "./components/LeftFootVlew";
import { RightFoot } from "./components/RightFootView";

import Toolbar from "@mui/material/Toolbar";
import Box from "@mui/material/Box";

type ViewElements = {
  tweet_id: string;
  author_id: string;
  created_at: string;
  text: string;
  name: string;
  username: string;
  profile_image_url: string;
  attachments: [string,string][];
};

function App() {
  const {
    focusTweetIdPair,
    tweetListPair,
    focusedPair,
    speechRatePair,
  } = React.useContext(AppContext);

  const [focusTweetId, setFocusTweetId] = focusTweetIdPair;
  const [tweetList, setTweetList] = tweetListPair;
  const [focused, setFocused] = focusedPair;
  const [speechRate, setSpeechRate] = speechRatePair;

  const scrollToFocus = (twid: string) => {
    const targetEl = document.getElementById(twid);
    if (targetEl && window.location.pathname === "/") {
      targetEl?.scrollIntoView({ behavior: "smooth" });
      console.log(twid);
    }
  };

  React.useEffect(() => {
    if (focused) {
      scrollToFocus(focusTweetId);
    }
  }, [focusTweetId, focused]);

  React.useEffect(() => {
    listen("tauri://frontend/token-register", (event) => {
      console.log(event);
      localStorage.setItem("token", JSON.stringify(event.payload));
    });

    listen("tauri://frontend/token-unregister", (event) => {
      console.log(event);
      localStorage.removeItem("token");
      exit(1);
    });

    listen("tauri://frontend/token-request", (_) => {
      const token = localStorage.getItem("token");
      if (token) {
        const json = JSON.parse(token);
        emit("tauri://backend/token-response", json);

        console.log(json);
      } else {
        emit("tauri://backend/token-response");

        console.log("return none");
      }
    });

    listen<ViewElements>("tauri://frontend/display/add", (event) => {
      const data: ViewElements = event.payload;
      tweetList.push({
        tweet_id: data.tweet_id,
        author_id: data.author_id,
        username: data.name,
        user_id: data.username,
        time: data.created_at,
        tweet: data.text,
        profile_image_url: data.profile_image_url,
        attachments: data.attachments,
      });
      setTweetList([...tweetList]);
    });

    listen<string>("tauri://frontend/display/delete", (event) => {
      const twid: string = event.payload;
      const index = tweetList.findIndex((elem) => elem.tweet_id === twid);
      tweetList.splice(index, 1);
      setTweetList([...tweetList]);
    });

    listen<string>("tauri://frontend/display/scroll", (event) => {
      const twid: string = event.payload;
      setFocusTweetId(twid);
      console.log(twid);
    });

    console.log("invoke setup_app function");

    invoke("set_speech_rate", { speechRate });

    invoke("setup_app").then(() => console.log("setup_app complete"));
    // 'emit, listen' works correct from here !!
    emit("tauri://backend/ipc-init");

    listen("tauri://frontend/speakers-ready", () => {
      console.log("tauri://frontend/speakers-ready");
      emit("tauri://backend/speakers-ready");
    });
  }, []);

  // Ignore right click on setting view
  document.addEventListener(
    "contextmenu",
    (event) => {
      console.log(event);
      event.preventDefault();
    },
    { capture: true }
  );

  return (
    <Box className="App">
      <BrowserRouter>
        <Toolbar />

        <Box sx={{ display: "flex" }}>
          <TWAppBar />

          <Box className="SideBar">
            <Drawer />
          </Box>

          <Box className="Body">
            <Routes>
              <Route path={`/`} element={<TweetView tweets={tweetList} />} />
              <Route path={`settings`} element={<Settings />} />
              <Route path={`licenses`} element={<Licenses />} />
            </Routes>
          </Box>
        </Box>

        <Box sx={{ display: "flex" }}>
          <Box className="LeftFoot ">
            <LeftFoot />
          </Box>
          <Box className="RightFoot">
            <RightFoot />
          </Box>
        </Box>
      </BrowserRouter>
    </Box>
  );
}

export default App;
