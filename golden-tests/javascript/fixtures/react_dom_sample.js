/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import {
  unstable_batchedUpdates,
  unstable_flushSync,
  unstable_createEventHandle,
} from 'react-dom';

const ReactDOM = {
  createPortal,
  findDOMNode,
  flushSync: unstable_flushSync,
  hydrate,
  render,
  unmountComponentAtNode,
  unstable_batchedUpdates,
  unstable_createEventHandle,
  unstable_renderSubtreeIntoContainer,
  version: React.version,
};

// Modern Root API
export {
  createRoot,
  hydrateRoot,
  unstable_createEventHandle,
} from './src/client/ReactDOMRoot';

// Utilities
export {
  findDOMNode,
  flushSync,
  hydrate,
  render,
  unmountComponentAtNode,
  unstable_batchedUpdates,
  unstable_renderSubtreeIntoContainer,
} from './src/client/ReactDOMLegacy';

// Server API
export {
  renderToString,
  renderToStaticMarkup,
} from './src/server/ReactDOMServerBrowser';

// Types
export type {
  Container,
  RootType,
  HydrationMode,
} from './src/client/ReactDOMRoot';

// Constants
const ROOT_ATTRIBUTE_NAME = 'data-reactroot';
const ELEMENT_NODE = 1;
const TEXT_NODE = 3;
const COMMENT_NODE = 8;
const DOCUMENT_NODE = 9;
const DOCUMENT_FRAGMENT_NODE = 11;

// Feature flags
const enableSchedulerDebugging = false;
const enableScopeAPI = false;
const enableCreateEventHandleAPI = false;
const enableFilterEmptyStringAttributesDOM = false;

// Helper functions
function getReactRootElementInContainer(container) {
  if (!container) {
    return null;
  }

  if (container.nodeType === DOCUMENT_NODE) {
    return container.documentElement;
  } else {
    return container.firstChild;
  }
}

function shouldHydrateDueToLegacyHeuristic(container) {
  const rootElement = getReactRootElementInContainer(container);
  return !!(
    rootElement &&
    rootElement.nodeType === ELEMENT_NODE &&
    rootElement.hasAttribute(ROOT_ATTRIBUTE_NAME)
  );
}

function isValidContainer(node) {
  return !!(
    node &&
    (node.nodeType === ELEMENT_NODE ||
      node.nodeType === DOCUMENT_NODE ||
      node.nodeType === DOCUMENT_FRAGMENT_NODE ||
      (node.nodeType === COMMENT_NODE &&
        node.nodeValue === ' react-mount-point-unstable '))
  );
}

// Legacy render function
function legacyRenderSubtreeIntoContainer(
  parentComponent,
  children,
  container,
  forceHydrate,
  callback,
) {
  if (!isValidContainer(container)) {
    throw new Error('Target container is not a DOM element.');
  }

  const isModernRoot =
    isContainerMarkedAsRoot(container) &&
    container._reactRootContainer === undefined;
  
  if (isModernRoot) {
    throw new Error(
      'You are calling ReactDOM.render() on a container that was previously ' +
      'passed to ReactDOM.createRoot(). This is not supported.',
    );
  }

  // TODO: Without `any` type, Flow says "Property cannot be accessed on any
  // member of intersection type." Whyyyyyy.
  let root = container._reactRootContainer;
  let fiberRoot;
  
  if (!root) {
    // Initial mount
    root = container._reactRootContainer = legacyCreateRootFromDOMContainer(
      container,
      forceHydrate,
    );
    fiberRoot = root._internalRoot;
    
    if (typeof callback === 'function') {
      const originalCallback = callback;
      callback = function() {
        const instance = getPublicRootInstance(fiberRoot);
        originalCallback.call(instance);
      };
    }
    
    // Initial mount should not be batched.
    unbatchedUpdates(() => {
      updateContainer(children, fiberRoot, parentComponent, callback);
    });
  } else {
    fiberRoot = root._internalRoot;
    
    if (typeof callback === 'function') {
      const originalCallback = callback;
      callback = function() {
        const instance = getPublicRootInstance(fiberRoot);
        originalCallback.call(instance);
      };
    }
    
    // Update
    updateContainer(children, fiberRoot, parentComponent, callback);
  }
  
  return getPublicRootInstance(fiberRoot);
}

// Modern event handling
const discreteEventPairsForSimpleEventPlugin = [
  ['blur', 'blur'],
  ['cancel', 'cancel'],
  ['click', 'click'],
  ['close', 'close'],
  ['contextmenu', 'contextMenu'],
  ['copy', 'copy'],
  ['cut', 'cut'],
  ['auxclick', 'auxClick'],
  ['dblclick', 'doubleClick'],
  ['dragend', 'dragEnd'],
  ['dragstart', 'dragStart'],
  ['drop', 'drop'],
  ['focus', 'focus'],
  ['input', 'input'],
  ['invalid', 'invalid'],
  ['keydown', 'keyDown'],
  ['keypress', 'keyPress'],
  ['keyup', 'keyUp'],
  ['mousedown', 'mouseDown'],
  ['mouseup', 'mouseUp'],
  ['paste', 'paste'],
  ['pause', 'pause'],
  ['play', 'play'],
  ['pointercancel', 'pointerCancel'],
  ['pointerdown', 'pointerDown'],
  ['pointerup', 'pointerUp'],
  ['ratechange', 'rateChange'],
  ['reset', 'reset'],
  ['seeked', 'seeked'],
  ['submit', 'submit'],
  ['touchcancel', 'touchCancel'],
  ['touchend', 'touchEnd'],
  ['touchstart', 'touchStart'],
  ['volumechange', 'volumeChange'],
];

// Event priorities
const DiscreteEvent = 0;
const UserBlockingEvent = 1;
const ContinuousEvent = 2;

// Event handling utilities
function getEventPriority(domEventName) {
  switch (domEventName) {
    case 'cancel':
    case 'click':
    case 'close':
    case 'contextmenu':
    case 'copy':
    case 'cut':
    case 'auxclick':
    case 'dblclick':
    case 'dragend':
    case 'dragstart':
    case 'drop':
    case 'focusin':
    case 'focusout':
    case 'input':
    case 'invalid':
    case 'keydown':
    case 'keypress':
    case 'keyup':
    case 'mousedown':
    case 'mouseup':
    case 'paste':
    case 'pause':
    case 'play':
    case 'pointercancel':
    case 'pointerdown':
    case 'pointerup':
    case 'ratechange':
    case 'reset':
    case 'seeked':
    case 'submit':
    case 'touchcancel':
    case 'touchend':
    case 'touchstart':
    case 'volumechange':
    case 'change':
    case 'selectionchange':
    case 'textInput':
    case 'compositionstart':
    case 'compositionend':
    case 'compositionupdate':
    case 'beforeblur':
    case 'afterblur':
    case 'beforeinput':
    case 'blur':
    case 'fullscreenchange':
    case 'focus':
    case 'hashchange':
    case 'popstate':
    case 'select':
    case 'selectstart':
      return DiscreteEvent;
    case 'drag':
    case 'dragenter':
    case 'dragexit':
    case 'dragleave':
    case 'dragover':
    case 'mousemove':
    case 'mouseout':
    case 'mouseover':
    case 'pointermove':
    case 'pointerout':
    case 'pointerover':
    case 'scroll':
    case 'toggle':
    case 'touchmove':
    case 'wheel':
    case 'mouseenter':
    case 'mouseleave':
    case 'pointerenter':
    case 'pointerleave':
      return ContinuousEvent;
    default:
      return UserBlockingEvent;
  }
}

// Error boundaries
class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true };
  }

  componentDidCatch(error, errorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return <h1>Something went wrong.</h1>;
    }

    return this.props.children;
  }
}

// Hooks
function useEvent(target, type, listener, options) {
  React.useEffect(() => {
    if (!target) {
      return;
    }
    
    target.addEventListener(type, listener, options);
    
    return () => {
      target.removeEventListener(type, listener, options);
    };
  }, [target, type, listener, options]);
}

// Concurrent features
const SyncLane = 0b0000000000000000000000000000001;
const SyncBatchedLane = 0b0000000000000000000000000000010;
const InputDiscreteHydrationLane = 0b0000000000000000000000000000100;
const InputDiscreteLane = 0b0000000000000000000000000001000;
const InputContinuousHydrationLane = 0b0000000000000000000000000010000;
const InputContinuousLane = 0b0000000000000000000000000100000;
const DefaultHydrationLane = 0b0000000000000000000000001000000;
const DefaultLane = 0b0000000000000000000000010000000;

// Scheduler integration
const ImmediatePriority = 99;
const UserBlockingPriority = 98;
const NormalPriority = 97;
const LowPriority = 96;
const IdlePriority = 95;

function runWithPriority(priority, fn) {
  const previousPriority = getCurrentPriority();
  
  try {
    setCurrentPriority(priority);
    return fn();
  } finally {
    setCurrentPriority(previousPriority);
  }
}

// Suspense
const SuspenseComponent = 13;
const SuspenseListComponent = 19;
const FundamentalComponent = 20;
const ScopeComponent = 21;
const OffscreenComponent = 22;
const LegacyHiddenComponent = 23;

// DevTools integration
if (typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ !== 'undefined') {
  const hook = __REACT_DEVTOOLS_GLOBAL_HOOK__;
  if (!hook.isDisabled) {
    hook.inject({
      bundleType: 1, // DEV
      version: React.version,
      rendererPackageName: 'react-dom',
    });
  }
}

// Export everything
export default ReactDOM;
export {
  ReactDOM as __SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED,
  ErrorBoundary,
  useEvent,
  getEventPriority,
  runWithPriority,
};