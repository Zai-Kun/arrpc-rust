// ==UserScript==
// @name         ReconnectButton
// @namespace    http://tampermonkey.net/
// @version      2024-03-25
// @description  try to take over the world!
// @author       Zai
// @match        *://*.discord.com/*
// @icon         https://www.google.com/s2/favicons?sz=64&domain=discord.com
// @grant        none
// ==/UserScript==

(function() {
    'use strict';
    let Dispatcher, lookupAsset, lookupApp, apps = {};

    let ws;
    let msg;

    async function getApplicationAsset(appID, key, lookupAsset) {
        if (/https?:\/\/(cdn|media)\.discordapp\.(com|net)\/attachments\//.test(key)) return "mp:" + key.replace(/https?:\/\/(cdn|media)\.discordapp\.(com|net)\//, "");
        return (await lookupAsset(appID, [key]))[0];
    }

    function connectWebSocket() {
        ws = new WebSocket('ws://127.0.0.1:1337');
        ws.onmessage = async x => {
            msg = JSON.parse(x.data);

            if (!Dispatcher) {
                console.log("Discpather not found")
                let wpRequire;
                window.webpackChunkdiscord_app.push([
                    [Symbol()], {},
                    x => wpRequire = x
                ]);
                window.webpackChunkdiscord_app.pop();

                const modules = wpRequire.c;

                for (const id in modules) {
                    const mod = modules[id].exports;

                    for (const prop in mod) {
                        const candidate = mod[prop];
                        try {
                            if (candidate && candidate.register && candidate.wait) {
                                Dispatcher = candidate;
                                break;
                            }
                        } catch {
                            continue;
                        }
                    }

                    if (Dispatcher) break;
                }
                const factories = wpRequire.m;
                for (const id in factories) {
                    if (factories[id].toString().includes('getAssetImage: size must === [number, number] for Twitch')) {
                        const mod = wpRequire(id);

                        lookupAsset = Object.values(mod).find(e => typeof e === 'function' && e.toString().includes('APPLICATION_ASSETS_FETCH_SUCCESS'));
                    }

                    if (lookupAsset) break;
                }
                for (const id in factories) {
                  if (factories[id].toString().includes('APPLICATION_RPC(')) {
                    const mod = wpRequire(id);

                    // fetchApplicationsRPC
                    const _lookupApp = Object.values(mod).find(e => {
                      if (typeof e !== 'function') return;
                      const str = e.toString();
                      return str.includes(',coverImage:') && str.includes('INVALID_ORIGIN');
                    });
                    if (_lookupApp) lookupApp = async appId => {
                      let socket = {};
                      await _lookupApp(socket, appId);
                      return socket.application;
                    };
                  }

                  if (lookupApp) break;
                }
            }

            if (msg.activity?.assets?.large_image) msg.activity.assets.large_image = await getApplicationAsset(msg.activity.application_id, msg.activity.assets.large_image, lookupAsset);
            if (msg.activity?.assets?.small_image) msg.activity.assets.small_image = await getApplicationAsset(msg.activity.application_id, msg.activity.assets.small_image, lookupAsset);
            
            if (msg.activity) {
                const appId = msg.activity.application_id;
                if (!apps[appId]) apps[appId] = await lookupApp(appId);

                const app = apps[appId];
                    if (!msg.activity.name) msg.activity.name = app.name;
            }

            Dispatcher.dispatch({
                type: "LOCAL_ACTIVITY_UPDATE",
                ...msg,
            });
        };

        ws.onclose = function(event) {
            if (Dispatcher) {
                Dispatcher.dispatch({
                    type: "LOCAL_ACTIVITY_UPDATE",
                    activity: null,
                    socketId: "0"
                });
            }
            let reconnectButton = document.getElementById('reconnectButton');
            if (reconnectButton) {
                reconnectButton.textContent = "🔴"
            }
        };
    }

    function reconnectWebSocket() {
        if (ws && ws.readyState !== WebSocket.CLOSED) {
            console.log("WebSocket is already open.");
            return;
        }
        connectWebSocket();
        let reconnectButton = document.getElementById('reconnectButton');
        if (reconnectButton) {
            reconnectButton.textContent = "🟢"
        }
    }

    function addButtonToToolbar() {
        const toolbar = document.querySelector('.toolbar_e44302');

        if (toolbar && !document.getElementById('reconnectButton')) {
            const reconnectButton = document.createElement("button");
            reconnectButton.id = 'reconnectButton';
            reconnectButton.textContent = (ws && ws.readyState !== WebSocket.CLOSED) ? "🟢" : "🔴";
            reconnectButton.style.marginLeft = "10px";
            reconnectButton.style.backgroundColor = "#5865F2";
            reconnectButton.style.color = "white";
            reconnectButton.style.border = "none";
            reconnectButton.style.padding = "5px 10px";
            reconnectButton.style.borderRadius = "4px";
            reconnectButton.style.cursor = "pointer";
            reconnectButton.style.fontSize = "16px";
            reconnectButton.style.fontWeight = "bold";
            reconnectButton.addEventListener("click", () => reconnectWebSocket());

            reconnectButton.onmouseover = function() {
                this.style.backgroundColor = "#4e5d94";
            };
            reconnectButton.onmouseout = function() {
                this.style.backgroundColor = "#5865F2";
            };

            toolbar.appendChild(reconnectButton);
        }
    }

    function observeToolbarChanges() {
        const targetNode = document.querySelector('#app-mount');

        if (!targetNode) {
            console.log('Discord app mount not found, retrying...');
            setTimeout(observeToolbarChanges, 5000);
            return;
        }

        const config = {
            childList: true,
            subtree: true
        };

        const callback = function(mutationsList, observer) {
            if (!document.getElementById('reconnectButton')) {
                addButtonToToolbar();
            }
        };

        const observer = new MutationObserver(callback);
        observer.observe(targetNode, config);
    }

    observeToolbarChanges();
})();
