import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useCallback,
  ReactNode,
} from 'react';
import { useLocation } from 'react-router-dom';
import { init, trackPageview, trackEvent } from '@plausible-analytics/tracker';
import { isDevEnv } from 'src/utils/dev';

const PLAUSIBLE_DOMAIN = 'cyb.ai';

type EventOptions = {
  props?: Record<string, string | number | boolean>;
  revenue?: { currency: string; amount: number };
};

type AnalyticsContextType = {
  trackEvent: (eventName: string, options?: EventOptions) => void;
};

const AnalyticsContext = createContext<AnalyticsContextType | null>(null);

function AnalyticsProvider({ children }: { children: ReactNode }) {
  // Initialize Plausible on mount
  useEffect(() => {
    if (isDevEnv()) {
      return undefined;
    }

    init({
      domain: PLAUSIBLE_DOMAIN,
      trackLocalhost: false,
      // Track outbound links automatically
      // Track file downloads automatically
    });

    // Track initial page view
    trackPageview();

    // Set up global click tracking
    const handleClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      const link = target.closest('a');

      if (link) {
        const href = link.getAttribute('href');

        // Track outbound links
        if (
          href &&
          (href.startsWith('http://') || href.startsWith('https://'))
        ) {
          const url = new URL(href);
          if (url.hostname !== window.location.hostname) {
            trackEvent('Outbound Link: Click', {
              props: { url: href },
            });
          }
        }

        // Track file downloads
        if (href) {
          const fileExtensions = [
            'pdf',
            'zip',
            'dmg',
            'exe',
            'doc',
            'docx',
            'xls',
            'xlsx',
            'ppt',
            'pptx',
            'csv',
            'mp3',
            'mp4',
          ];
          const extension = href.split('.').pop()?.toLowerCase();
          if (extension && fileExtensions.includes(extension)) {
            trackEvent('File Download', {
              props: { url: href, extension },
            });
          }
        }
      }

      // Track button clicks with data-analytics attribute
      const analyticsElement = target.closest('[data-analytics]');
      if (analyticsElement) {
        const eventName = analyticsElement.getAttribute('data-analytics');
        if (eventName) {
          trackEvent(eventName);
        }
      }
    };

    document.addEventListener('click', handleClick);

    return () => {
      document.removeEventListener('click', handleClick);
    };
  }, []);

  // Track page views on route changes
  const location = useLocation();

  useEffect(() => {
    if (isDevEnv()) {
      return;
    }

    // Skip initial render (already tracked above)
    const isInitialRender = !document.referrer && location.key === 'default';
    if (!isInitialRender) {
      trackPageview();
    }
  }, [location.pathname, location.search]);

  const handleTrackEvent = useCallback(
    (eventName: string, options?: EventOptions) => {
      if (isDevEnv()) {
        console.log('[Analytics Dev]', eventName, options);
        return;
      }
      trackEvent(eventName, options);
    },
    []
  );

  const value = useMemo(
    () => ({
      trackEvent: handleTrackEvent,
    }),
    [handleTrackEvent]
  );

  return (
    <AnalyticsContext.Provider value={value}>
      {children}
    </AnalyticsContext.Provider>
  );
}

function useAnalytics() {
  const context = useContext(AnalyticsContext);
  if (!context) {
    throw new Error('useAnalytics must be used within AnalyticsProvider');
  }
  return context;
}

export { AnalyticsProvider, useAnalytics };
export default AnalyticsProvider;
