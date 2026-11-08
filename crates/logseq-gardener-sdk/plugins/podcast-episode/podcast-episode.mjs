// lsg first-party plugin podcast-episode: a player that keeps an episode's recording playing
// while the reader moves between pages. The build gives each episode page an entry with
// `recording` (a URL relative to the site root) and `title`. On the first episode page the
// module moves that page's inline <audio> into a player outside <main>; on a later episode
// with another recording, a button takes the player over only when the reader starts it.
// Dialogue timestamps (`lsg:media-seek`) seek the player. lsg/Ref/First-Party Plugins, in
// docs/pages/lsg___Ref___First-Party Plugins.md, documents the plugin.

/** `value` as an absolute URL without its fragment, or null. */
export function recordingURL(value, base) {
  try {
    const url = new URL(value, base);
    url.hash = '';
    return url.href;
  } catch {
    return null;
  }
}

/** `seconds`, kept within the recording when its length is known. */
export function clampTime(seconds, duration) {
  return Number.isFinite(duration) && duration >= 0 ? Math.min(seconds, duration) : seconds;
}

/** The time a `lsg:media-seek` request asks for when it names `source`, otherwise null. */
export function requestedTime(detail, source) {
  const seconds = detail?.seconds;
  if (!Number.isSafeInteger(seconds) || seconds < 0 || typeof detail.source !== 'string') return null;
  return detail.source === source ? seconds : null;
}

/** The episode an adoption describes, or null on a page without this plugin's entry. */
export function episodeOf(adoption, base) {
  const { recording, title } = adoption.metadata ?? {};
  const source = typeof recording === 'string' ? recordingURL(recording, base) : null;
  if (!source) return null;
  return {
    source,
    title: typeof title === 'string' && title ? title : adoption.page.title,
    url: new URL(adoption.page.url, base).href,
  };
}

/** The page's inline <audio> for `source`, outside references and embeds. */
export function inlineAudio(main, source, base) {
  for (const audio of main?.querySelectorAll('audio') ?? []) {
    if (audio.closest('.references, .embed-block, .embed-page')) continue;
    if (recordingURL(audio.getAttribute('src'), base) === source) return audio;
  }
  return null;
}

export default {
  id: 'podcast-episode',
  apiVersion: 1,
  setup(host) {
    const doc = host.document;
    const base = host.baseURL;
    let region = null, footer = null, audio = null, link = null, status = null;
    let playing = null; // the episode the player holds
    let here = null; // the episode of the page on screen
    let pending = null;
    let offer = null; // the button that starts the page's own episode in the player
    let origin = null; // where the player's <audio> came from, to put it back on cleanup

    const say = (text) => { if (status) status.textContent = text; };
    const applySeek = () => {
      if (pending === null || audio.readyState < 1) return;
      try {
        audio.currentTime = clampTime(pending, audio.duration);
        pending = null;
      } catch {
        say('Unable to seek. Use the recording controls to try again.');
      }
    };
    const play = () => {
      say('');
      // play() runs inside the reader's click, before the recording's metadata exists.
      try {
        Promise.resolve(audio.play()).catch(() => say('Press play in the recording controls to listen.'));
      } catch {
        say('Press play in the recording controls to listen.');
      }
    };
    const show = (episode) => {
      playing = episode;
      link.href = episode.url;
      link.textContent = episode.title;
    };
    function build(episode, element) {
      region = host.persistentRegion('player');
      footer = doc.createElement('footer');
      footer.className = 'lsg-episode-player';
      footer.setAttribute('aria-label', 'Episode recording');
      link = doc.createElement('a');
      link.className = 'lsg-episode-player-title';
      status = doc.createElement('span');
      status.className = 'lsg-episode-player-status';
      status.setAttribute('role', 'status');
      status.setAttribute('aria-live', 'polite');
      audio = element;
      origin = { parent: element.parentNode, next: element.nextSibling };
      audio.controls = true;
      // A relative src would resolve against each later page's address; an unloaded recording
      // can take the absolute URL without interrupting anything.
      if (audio.paused && audio.readyState === 0) audio.setAttribute('src', episode.source);
      audio.addEventListener('loadedmetadata', applySeek);
      audio.addEventListener('durationchange', applySeek);
      audio.addEventListener('error', () => { pending = null; say('The recording could not load.'); });
      const details = doc.createElement('div');
      details.className = 'lsg-episode-player-details';
      details.append(link, status);
      footer.append(details, audio);
      region.element.append(footer);
      doc.body.classList.add('lsg-has-episode-player');
      show(episode);
    }
    // Play `episode` from `seconds`; another episode's recording replaces the current one.
    function activate(episode, seconds) {
      if (playing?.source !== episode.source) {
        audio.pause();
        audio.setAttribute('src', episode.source);
        audio.load();
        show(episode);
        offer?.remove();
      }
      pending = seconds;
      applySeek();
      play();
    }
    // Show the page on screen: its inline recording moves into the player, disappears when the
    // player already holds it, or becomes a button that starts it in the player.
    function adopt(episode) {
      if (!episode) return;
      const element = inlineAudio(host.document.querySelector('main'), episode.source, base);
      if (!element || element === audio) return;
      if (!audio) return build(episode, element);
      const item = element.closest('li');
      if (playing.source === episode.source) {
        element.remove();
      } else {
        offer = doc.createElement('button');
        offer.type = 'button';
        offer.className = 'lsg-episode-play';
        offer.textContent = `Play ${episode.title}`;
        offer.addEventListener('click', () => activate(episode, 0));
        element.replaceWith(offer);
      }
      item?.classList.add('lsg-episode-recording-origin');
    }
    const stop = host.onPageAdopted((adoption) => {
      offer = null;
      here = episodeOf(adoption, base);
      adopt(here);
    });
    const seek = (event) => {
      if (!here || !audio || event.defaultPrevented || !event.cancelable) return;
      const seconds = requestedTime(event.detail, here.source);
      if (seconds === null) return;
      event.preventDefault();
      activate(here, seconds);
    };
    doc.addEventListener('lsg:media-seek', seek);
    return () => {
      stop();
      doc.removeEventListener('lsg:media-seek', seek);
      if (!audio) return;
      audio.pause();
      // The recording returns to its page, so the page works as it does without the plugin.
      if (origin.parent?.isConnected) origin.parent.insertBefore(audio, origin.next?.parentNode === origin.parent ? origin.next : null);
      doc.body.classList.remove('lsg-has-episode-player');
      region.dispose();
    };
  },
};
