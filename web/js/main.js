'use strict';

import PeerSession from "./peer-session.js";

var peerSession = null;

const localVideo = document.getElementById('localVideo');
const remoteVideo = document.getElementById('remoteVideo');

const joinButton = document.getElementById('joinButton');
joinButton.onclick = async () => {
  if (peerSession) {
    return;
  }

  const username = document.getElementById('nameInput').value;
  peerSession = new PeerSession(username, "ws://127.0.0.1:8080/ws");
  peerSession.onvideounmute = handleOnvideounmute;
  peerSession.onclose = handleOnclose;

  try {
    await addCameraMic(peerSession);
  } catch (err) {
    console.error(err);
  }
};

const leaveButton = document.getElementById('leaveButton');
leaveButton.onclick = async () => {
  if (!peerSession) {
    return;
  }

  peerSession.close();
}

function handleOnvideounmute(stream) {
  if (remoteVideo.srcObject) {
    return;
  }

  remoteVideo.srcObject = stream;
}

function handleOnclose() {
  localVideo.srcObject.getTracks().forEach(t => t.stop());
  remoteVideo.srcObject.getTracks().forEach(t => t.stop());

  localVideo.removeAttribute("src");
  localVideo.removeAttribute("srcObject");
  remoteVideo.removeAttribute("src");
  remoteVideo.removeAttribute("srcObject");

  peerSession = null;
}

async function addCameraMic(peerSession) {
  try {
    const constraints = { video: true, audio: true };
    const stream = await navigator.mediaDevices.getUserMedia(constraints);

    peerSession.addMediaStream(stream);
    localVideo.srcObject = stream;
  } catch (err) {
    console.error(err);
  }
}
