import React, { useState } from 'react';
import clsx from 'clsx';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './styles.module.css';
import CodeBlock from '@theme/CodeBlock';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useIsBrowser from '@docusaurus/useIsBrowser';

const os = () => {
  const isBrowser = useIsBrowser();
  const platform = isBrowser ? navigator.platform : '';
  if (platform.substring('Mac') != 1) {
    return ['MacOS', `curl -fsSL https://nanobus.io/install.sh | /bin/bash`];
  }
  if (platform.substring('Linux') != 1) {
    return ['Linux', `wget -q https://nanobus.io/install.sh -O - | /bin/bash`];
  }
  if (platform.substring('windows') != 1) {
    return [
      'Windows',
      `powershell -Command "iwr -useb https://nanobus.io/install.ps1 | iex"`,
    ];
  }
  return undefined;
};

function CLIInstall() {
  return (
    <BrowserOnly fallback={<div>simple</div>}>
      {() => {
        const cmd = os();
        if (!cmd) {
          return <div />;
        }

        return (
          <div>
            <h4>{cmd[0]} Install</h4>
            <CodeBlock className="codeBlock" language="shell">
              {cmd[1]}
            </CodeBlock>
          </div>
        );
      }}
    </BrowserOnly>
  );
}

export default function Home() {
  const context = useDocusaurusContext();
  const { siteConfig = {} } = context;

  return (
    <Layout title={`${siteConfig.title}`} description={siteConfig.tagline}>
      <main className="HomePage">
        {/* HEADER */}
        <header className={clsx('hero full', styles.heroBanner)}>
          <div className="container">
            <div className="row">
              <div className="col col--2">{/* Example */}</div>
              <div className="col col--8">
                <h2 className="hero__title">{siteConfig.tagline}</h2>
                <p className="hero__subtitle">
                  NanoBus is a lightweight framework for building secure and
                  scalable software services.
                </p>
                <div>
                  <Link
                    className={clsx(
                      'button hero--button button--md button--primary responsive-button',
                      styles.button
                    )}
                    to={useBaseUrl('/getting-started')}
                    style={{ marginTop: 10 }}
                  >
                    Get Started →
                  </Link>
                  <Link
                    className={clsx(
                      'button hero--button button--md button--secondary button--outline responsive-button',
                      styles.button
                    )}
                    to={useBaseUrl('/components')}
                    style={{ marginTop: 10 }}
                  >
                    Explore Components
                  </Link>

                  <CLIInstall />
                </div>
              </div>
              <div className="col col--2">{/* Example */}</div>
            </div>
          </div>
        </header>
      </main>
    </Layout>
  );
}
