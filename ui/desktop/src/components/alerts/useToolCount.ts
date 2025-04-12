import { useState, useEffect } from 'react';
import { getTools } from '../../api';

const { clearTimeout } = window;

export const useToolCount = (triggerRefetch?: unknown) => {
  const [toolCount, setToolCount] = useState<number | null>(null);

  useEffect(() => {
    let retryCount = 0;
    const maxRetries = 5;
    const initialDelay = 1000;
    let timeoutId: ReturnType<typeof setTimeout>;

    const fetchTools = async () => {
      try {
        const response = await getTools();
        // Hack to get around getTools returning an empty list if the engine is not ready yet
        if (!response.error && response.data) {
          // If we get a valid response with no tools, update immediately
          if (response.data.length === 0) {
            if (retryCount < maxRetries) {
              // Only retry if we haven't hit max retries
              retryCount++;
              const delay = initialDelay * Math.pow(2, retryCount - 1);
              console.log(
                `Got empty tool list, retrying (${retryCount}/${maxRetries}) in ${delay}ms...`
              );
              // Set the current count to 0
              setToolCount(0);
              timeoutId = setTimeout(fetchTools, delay);
            } else {
              // Max retries reached, confirm zero tools
              setToolCount(0);
              clearTimeout(timeoutId);
            }
          } else {
            // We got tools, update the count and clear any pending retries
            console.log(`Got tool count: ${response.data.length} (after ${retryCount} retries)`);
            setToolCount(response.data.length);
            clearTimeout(timeoutId);
          }
        } else {
          setToolCount(0);
          clearTimeout(timeoutId);
        }
      } catch (err) {
        console.error('Error fetching tools:', err);
        setToolCount(0);
        clearTimeout(timeoutId);
      }
    };

    fetchTools();

    // Cleanup timeouts on unmount or when triggerRefetch changes
    return () => {
      clearTimeout(timeoutId);
    };
  }, [triggerRefetch]);

  return toolCount;
};
