import React from "react";
import "./App.css";

import { Routes, Route, useLocation } from "react-router-dom";

import { invoke } from "@tauri-apps/api";
import { listen, emit } from "@tauri-apps/api/event";
import { exit } from "@tauri-apps/api/process";

import { AppContext } from "./AppContext";
import { Licenses } from "./components/LicenseView";
import { Drawer } from "./components/DrawerView";
import { TweetView } from "./components/TweetView";
import { SearchView } from "./components/SearchView";
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
    searchTweetListPair,
    focusedPair,
    skippedPair,
    speechRatePair,
  } = React.useContext(AppContext);

  const [focusTweetId, setFocusTweetId] = focusTweetIdPair;
  const [tweetList, setTweetList] = tweetListPair;
  const [searchFocusTweetId, setSearchFocusTweetId] = focusTweetIdPair;
  const [searchTweetList, setSearchTweetList] = searchTweetListPair;
  const [skipped, setSkipped] = skippedPair;
  const [focused, setFocused] = focusedPair;
  const [speechRate, setSpeechRate] = speechRatePair;

  React.useEffect(() => {
    if (focused) {
        if (location.pathname === "/") {
          scrollToFocus(focusTweetId);
        } else if (location.pathname === "/search") {
          scrollToFocus(searchFocusTweetId);
        }
    }
  }, [focusTweetId, searchFocusTweetId, focused]);

  const location = useLocation();
  React.useEffect(() => {
    console.log(location.pathname);

    if (location.pathname === "/") {
        invoke("set_timeline", {"timeline": "User"} );
        invoke("set_timeline_view", {"timeline": "User"} );
    } else if (location.pathname === "/search") {
        // set_timeline is called when the search button is pushed.
        invoke("set_timeline_view", {"timeline": {"Search": {"query": ""}}} );
    }

    if (location.pathname === "/") {
      scrollToFocus(focusTweetId);
    } else if (location.pathname === "/search") {
      scrollToFocus(searchFocusTweetId);
    }

  }, [location]);

  const scrollToFocus = (twid: string) => {
    const targetEl = document.getElementById(twid);
    if (targetEl
        && location.pathname === "/"
        || location.pathname === "/search") {
      targetEl?.scrollIntoView({ behavior: "smooth" });
      console.log(twid);
    }
  };

  React.useEffect(() => {
    if (skipped) {
      console.log("skipped");

      let id;
      if (location.pathname === "/") {
        const index = tweetList.findIndex((elem) => elem.tweet_id === focusTweetId);
        if (index in tweetList) {
          id = tweetList[index + 1]?.tweet_id;
        } else {
          id = "";
        }
      } else if (location.pathname === "/search") {
        const index = searchTweetList.findIndex((elem) => elem.tweet_id === searchFocusTweetId);
        if (index in searchTweetList) {
          id = searchTweetList[index + 1]?.tweet_id;
        } else {
          id = "";
        }
      }

      invoke('jump', {twid: id});

      setSkipped(false);
    }
  }, [skipped]);

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

    listen<ViewElements>("tauri://frontend/display/user/add", (event) => {
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

    listen<string>("tauri://frontend/display/user/delete", (event) => {
      const twid: string = event.payload;
      const index = tweetList.findIndex((elem) => elem.tweet_id === twid);
      tweetList.splice(index, 1);
      setTweetList([...tweetList]);
    });

    listen<string>("tauri://frontend/display/user/scroll", (event) => {
      const twid: string = event.payload;
      setFocusTweetId(twid);
      console.log(twid);
    });

    listen<ViewElements>("tauri://frontend/display/search/add", (event) => {
      const data: ViewElements = event.payload;
      searchTweetList.push({
        tweet_id: data.tweet_id,
        author_id: data.author_id,
        username: data.name,
        user_id: data.username,
        time: data.created_at,
        tweet: data.text,
        profile_image_url: data.profile_image_url,
        attachments: data.attachments,
      });
      setSearchTweetList([...searchTweetList]);
    });

    listen<string>("tauri://frontend/display/search/delete", (event) => {
      const twid: string = event.payload;
      const index = searchTweetList.findIndex((elem) => elem.tweet_id === twid);
      searchTweetList.splice(index, 1);
      setSearchTweetList([...searchTweetList]);
    });

    listen<string>("tauri://frontend/display/search/scroll", (event) => {
      const twid: string = event.payload;
      setSearchFocusTweetId(twid);
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
        <Toolbar />

        <Box sx={{ display: "flex" }}>
          <TWAppBar />

          <Box className="SideBar">
            <Drawer />
          </Box>

          <Box className="Body">
            <Routes>
              <Route path={`/`} element={<TweetView tweets={tweetList} />} />
              <Route path={`search`} element={<SearchView tweets={searchTweetList} />} />
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
    </Box>
  );
}

export default App;
