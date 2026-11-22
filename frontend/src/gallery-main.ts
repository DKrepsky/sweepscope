import './design-system/tokens.css';
import './design-system/base.css';
import { mount } from 'svelte';
import Gallery from './pages/gallery/Gallery.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('Missing #app mount point');
mount(Gallery, { target });
